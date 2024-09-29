#include <stdio.h>
#include <stdlib.h>

typedef unsigned char *byte_pointer;
void show_bytes(byte_pointer start, int len)
{
    size_t i;
    for (i = 0; i < len; i++)
    {
        printf("A:%02x ", start[i]);
    }
    printf("\n");
    for (int i = len - 1; i >= 0; i--)
    {
        printf("B:%02x ", start[i]);
    }
    printf("\n");
}

void show_int(int x)
{
    show_bytes((byte_pointer)&x, sizeof(int));
}

void show_float(float x)
{
    show_bytes((byte_pointer)&x, sizeof(float));
}

void show_pointer(void *x)
{
    show_bytes((byte_pointer)&x, sizeof(void *));
}

int main()
{
    int a = 123;
    show_int(a);

    float b = 3.14;
    show_float(b);

    int *c = &a;
    show_pointer(c);

    int *p = &a;
    typeof(c) *h = &p;
    show_pointer(h);

    int e = 5;
    int *d = &e;
    printf("%d\n", *d);
    (*d)++;
    printf("%d\n", *d);

    printf("Practice...........................\n");
    int x = 0x87654321;
    byte_pointer valp = (byte_pointer)&x;
    show_bytes(valp, 1);
    show_bytes(valp, 2);
    show_bytes(valp, 3);
    show_bytes(valp, 4);

    int y = 1;
    int *p2 = &y;
    printf("%p\n", p2);

    return 0;
}
