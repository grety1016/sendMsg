#![allow(dead_code)]
#[macro_use]
extern crate tracing;
#[allow(unused_imports)]
use std::{
    any::{Any, TypeId},
    fmt::{self, Debug, Display},
    future::Future,
    marker::PhantomData,
    ops::Deref,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    thread::JoinHandle,
    time::Duration,
};
use tokio::{
    runtime,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task,
};

/// 定义主题
#[macro_export]
macro_rules! define_topic {
    ($($(#[$attrs:meta])* [$name:literal] $topic:ident:$ty:ty;)*) => {
        $(
            $(#[$attrs])*
            #[doc="\n`"]
            #[doc=$name]
            #[doc="`"]
            #[allow(non_camel_case_types)]
            pub struct $topic;

            impl $crate::Topic for $topic {
                type Message = $ty;
                const NAME: &'static str = $name;
            }
        )*
    };
}

/// 消息主题
pub trait Topic: Any + Sized + Send + 'static {
    type Message: Any + Sized + Send + 'static;
    const NAME: &'static str;
}

/// 订阅ID
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SubscriberId(u64);

/// 主题消息
pub struct Message<T: Topic> {
    pack: MessagePack,
    _marker: PhantomData<T>,
}

impl<T> Deref for Message<T>
where
    T: Topic,
{
    type Target = T::Message;
    fn deref(&self) -> &Self::Target {
        self.pack.as_ref::<T>()
    }
}

#[allow(noop_method_call)]
impl<T> Debug for Message<T>
where
    T: Topic,
    T::Message: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.deref().deref(), f)
    }
}
#[allow(noop_method_call)]

impl<T> Display for Message<T>
where
    T: Topic,
    T::Message: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.deref().deref(), f)
    }
}

/// 事件分发器
pub struct Eventful {
    next_id: AtomicU64,
    tx: UnboundedSender<Event>,
    thrd_hdl: Option<JoinHandle<()>>,
}

impl Eventful {
    pub fn new() -> Eventful {
        let (tx, rx) = mpsc::unbounded_channel();
        let thrd_hdl =
            thread::Builder::new().name("eventful".into()).spawn(move || Self::dispatch(rx)).unwrap();
        let next_id = AtomicU64::new(0);
        Eventful {
            next_id,
            tx,
            thrd_hdl: Some(thrd_hdl),
        }
    }
    /// 订阅事件
    pub fn subscribe<T, F>(&self, topic: T, handler: F) -> SubscriberId
    where
        T: Topic,
        F: FnMut(Message<T>) + Send + 'static,
    {
        let sub_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let sub_id = SubscriberId(sub_id);
        let sub = Subscriber::new(sub_id, topic, handler);
        self.tx.send(Event::Subscribe(sub)).expect("channel was closed");
        sub_id
    }
    /// 订阅事件（异步回调版本）
    pub fn subscribe_async<T, F, Fut>(&self, topic: T, handler: F) -> SubscriberId
    where
        T: Topic,
        F: FnMut(Message<T>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let sub_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let sub_id = SubscriberId(sub_id);
        let sub = Subscriber::new_async(sub_id, topic, handler);
        self.tx.send(Event::Subscribe(sub)).expect("channel was closed");
        sub_id
    }
    /// 取消订阅事件
    ///
    /// 不指定`sub_id`时取消指定主题的所有订阅者
    pub fn unsubscribe<T>(&self, topic: T, sub_id: impl Into<Option<SubscriberId>>)
    where
        T: Topic,
    {
        let _ = self.tx.send(Event::Unsubscribe(UnsubscribeEvent::new(topic, sub_id.into())));
    }
    /// 发布消息
    pub fn publish<T>(&self, topic: T, msg: T::Message)
    where
        T: Topic,
    {
        self.tx.send(Event::Publish(PublishEvent::new(topic, msg))).expect("channel was closed");
    }
    /// 消息通道
    pub fn shutdown(mut self) {
        self.tx.send(Event::Shutdown).expect("channel was closed");
        let _ = self.thrd_hdl.take().unwrap().join();
    }
    /// 监听并派发消息
    fn dispatch(mut rx: UnboundedReceiver<Event>) {
        let run_loop = async move {
            let mut subscribers = Vec::new();
            while let Some(msg) = rx.recv().await {
                match msg {
                    Event::Subscribe(subscriber) => {
                        trace!("subscribe: {}", subscriber.topic_name);
                        subscribers.push(subscriber)
                    },
                    Event::Unsubscribe(evt) => {
                        trace!("unsubscribe: {}, unsub_id: {:?}", evt.topic_name, evt.sub_id);
                        if let Some(sub_id) = evt.sub_id {
                            //取消订阅指定ID的订阅者
                            let idx = subscribers
                                .iter()
                                .position(|sub| sub.topic_id == evt.topic_id && sub.id == sub_id);
                            if let Some(idx) = idx {
                                subscribers.remove(idx);
                            }
                        } else {
                            //取消订阅指定主题的所有订阅者
                            let mut idx = subscribers.len();
                            while idx > 0 {
                                idx -= 1;
                                if subscribers.get(idx).unwrap().topic_id == evt.topic_id {
                                    subscribers.remove(idx);
                                }
                            }
                        };
                    },
                    Event::Publish(evt) => {
                        trace!("publish: {}", evt.topic_name);
                        let (topic_id, topic_msg) = evt.unpack();
                        for sub in subscribers.iter_mut() {
                            if sub.topic_id == topic_id {
                                (sub.handler)(topic_msg.clone());
                            }
                        }
                    },
                    Event::Shutdown => {
                        trace!("shutdown");
                        break;
                    },
                }
            }
        };
        //单线程运行时
        let rt = runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let local = task::LocalSet::new();
        //运行
        local.spawn_local(run_loop);
        rt.block_on(local);
        trace!("exit");
    }
}

impl Drop for Eventful {
    fn drop(&mut self) {
        if let Some(handle) = self.thrd_hdl.take() {
            let _ = self.tx.send(Event::Shutdown);
            let _ = handle.join();
        }
    }
}

/// 框架事件消息
#[derive(Debug)]
enum Event {
    Subscribe(Subscriber),
    Unsubscribe(UnsubscribeEvent),
    Publish(PublishEvent),
    Shutdown,
}

/// 取消订阅事件
#[derive(Debug)]
struct UnsubscribeEvent {
    topic_id: TypeId,
    topic_name: &'static str,
    sub_id: Option<SubscriberId>,
}

impl UnsubscribeEvent {
    fn new<T: Topic>(_: T, sub_id: Option<SubscriberId>) -> Self {
        UnsubscribeEvent {
            topic_id: TypeId::of::<T>(),
            topic_name: T::NAME,
            sub_id,
        }
    }
}

/// 发布主题事件
struct PublishEvent {
    topic_id: TypeId,
    topic_name: &'static str,
    msg: Box<dyn Any + Send + 'static>,
}

impl PublishEvent {
    fn new<T: Topic>(_: T, msg: T::Message) -> Self {
        PublishEvent {
            topic_id: TypeId::of::<T>(),
            topic_name: T::NAME,
            msg: Box::new(msg),
        }
    }
    /// 解包
    fn unpack(self) -> (TypeId, MessagePack) {
        (self.topic_id, MessagePack(self.msg.into()))
    }
}

impl Debug for PublishEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "publish pack")
    }
}

/// 引用计数的用户消息
#[derive(Clone)]
struct MessagePack(Rc<dyn Any + Send + 'static>);

impl MessagePack {
    fn as_ref<T: Topic>(&self) -> &T::Message {
        //SAFETY 消息载体只有主题类型匹配时才会解引用
        self.0.downcast_ref::<T::Message>().unwrap()
    }
}

impl<T: Topic> Into<Message<T>> for MessagePack {
    fn into(self) -> Message<T> {
        Message {
            pack: self,
            _marker: PhantomData,
        }
    }
}

/// 订阅者
struct Subscriber {
    id: SubscriberId,
    topic_id: TypeId,
    topic_name: &'static str,
    handler: Box<dyn FnMut(MessagePack) + Send + 'static>,
}

impl Subscriber {
    fn new<T, F>(id: SubscriberId, _: T, mut handler: F) -> Self
    where
        T: Topic,
        F: FnMut(Message<T>) + Send + 'static,
    {
        //特化回调事件
        let handler = move |msg: MessagePack| {
            handler(msg.into());
        };
        Subscriber {
            id,
            topic_id: TypeId::of::<T>(),
            topic_name: T::NAME,
            handler: Box::new(handler),
        }
    }
    /// 异步回调版本
    fn new_async<T, F, Fut>(id: SubscriberId, _: T, mut handler: F) -> Self
    where
        T: Topic,
        F: FnMut(Message<T>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        //特化回调事件
        let handler = move |msg: MessagePack| {
            let fut = handler(msg.into());
            task::spawn_local(fut);
        };
        Subscriber {
            id,
            topic_id: TypeId::of::<T>(),
            topic_name: T::NAME,
            handler: Box::new(handler),
        }
    }
}

impl Debug for Subscriber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id: {:?}, topic_id: {:?}, topic_name: {}", self.id, self.topic_id, self.topic_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicI16, Ordering},
        Arc,
    };

    fn init() {
        let subscriber =
            tracing_subscriber::FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    define_topic! {
        /// 主题A
        ["topic A"]
        TopicA: String;
        /// 主题B
        ["topic B"]
        TopicB: i32;
        /// 主题C
        ["topic C"]
        TopicC: ();
    }

    #[test]
    fn should_works() {
        init();

        let flag = Arc::new(AtomicI16::new(0));

        let eventful = Eventful::new();
        eventful.subscribe(TopicA, {
            let flag = flag.clone();
            move |msg| {
                info!("sub1: {}", msg);
                flag.store(1, Ordering::Relaxed);
            }
        });

        let sub_id = eventful.subscribe(TopicA, {
            let flag = flag.clone();
            move |msg| {
                info!("sub2: {}", msg);
                flag.store(2, Ordering::Relaxed);
            }
        });

        eventful.publish(TopicA, "abcd".to_owned());
        eventful.unsubscribe(TopicA, sub_id);
        //eventful.unsubscribe(TopicA, None);
        eventful.publish(TopicA, "xxxxx".to_owned());

        eventful.subscribe_async(TopicB, {
            let flag = flag.clone();
            move |msg| {
                let flag = flag.clone();
                async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    info!("sub3: {}", msg);
                    flag.store(3, Ordering::Relaxed);
                }
            }
        });
        eventful.publish(TopicB, 1232323);

        eventful.subscribe_async(TopicC, {
            let flag = flag.clone();
            move |_| {
                let flag = flag.clone();
                async move {
                    info!("sub4");
                    flag.store(4, Ordering::Relaxed);
                }
            }
        });
        eventful.publish(TopicC, ());

        eventful.shutdown();

        info!("flag: {}", flag.load(Ordering::Relaxed));

        //assert!(flag.load(Ordering::Relaxed) == 4);
    }
}
