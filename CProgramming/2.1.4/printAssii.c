#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

void printChars(const char *str)
{
    while (1)
    {
        printf("%0.2x  ", *str);
        if (*str == '\0')
        {
            break;
        }
        str++;
    }
    printf("\n");
}

int sum(int x, int y)
{
    return x + y;
}

int main()
{
    const char *a = "abcdef";
    printf("str's length: %d\n", strlen(a));
    printChars(a);
    show_bytes((byte_pointer)a, strlen(a));

    int (*ptrfn)(int, int);
    ptrfn = sum;

    show_bytes((byte_pointer)ptrfn, sizeof(ptrfn));

    int d = 123456;
    int b = 123456;
    int c = 123456;
    printf("address: %p \n",&d);
    printf("address: %p \n",&b);
    printf("address: %p \n",&c);

    show_int(d);
    show_int(b);
    show_int(c);
}