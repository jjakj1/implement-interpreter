# Monkey 解释器

该语言拥有如下特性：

* 类 C 语法
* 变量绑定
* 整型和 bool 值
* 算术运算符
* 内置函数
* 头等函数和高阶函数
* 闭包
* 字符串数据结构
* 数组数据结构
* 哈希数据结构

以下是一个语言使用案例：

```
let age = 1;
let name = "Monkey";
let result = 10 * (20 / 2);
let myArray = [1, 2, 3, 4];
let messi = {"name": "Messi", "age": "36"};
let add = fn(a, b) { a + b; };
```

该解释器包括以下部分，并按照顺序实现：

* 词法分析器
* 语法分析器
* 抽象语法树（AST）
* 内部求值器
* 求值器

使用 `cargo run` 可以在本地运行该解释器
