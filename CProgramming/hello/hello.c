#include <stdio.h>
#include <stdlib.h>

int main()
{
    char str[] = "Hello,World!";
    int i = 0;
    printf("CharacterLength: %d\n", sizeof(str));
    // 打印每个字符的ASCII码
    while (i < sizeof(str))
    {
        printf("Character: %c, ASCII: %d\n", str[i], str[i]);
        i++;
    }

    char *c_ptr = (char *)malloc(4 * sizeof(char));
    if (c_ptr == NULL)
    {
        printf("Memory allocation failed\n");
        return 1;
    }
    c_ptr[0] = 'B';
    c_ptr[1] = 'i';
    c_ptr[2] = 'g';
    c_ptr[3] = '\0';
    printf("Address: %p\n", (void *)c_ptr);
 

    return 0;
}