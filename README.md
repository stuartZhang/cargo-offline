# `cargo-offline`命令

`cargo-offline`是标准`cargo`命令的包装器。其被用来，根据距离`cargo-offline`命令执行目录最近的`Cargo.toml`文件是否曾经被修改，来给被包装的`cargo`命令智能地添加`--offline`命令行参数（即，离线编译）。

## 动机

最近一段时间[github.com](https://github.com)的可访问稳定性实在是**很差**。但是，`cargo`命令（无论执行它的哪个子命令）都要求首先同步[crates.io-index](https://github.com/rust-lang/crates.io-index)索引清单。于是，日常的开发与编译工作流就被时不时地被阻塞于

```shell
warning: spurious network error (1 tries remaining): [35] SSL connect error (schannel: failed to receive handshake, SSL/TLS connection failed); class=Net (12)
Caused by:
  Unable to update registry `crates-io`
Caused by:
  failed to fetch `https://github.com/rust-lang/crates.io-index`
Caused by:
  [35] SSL connect error (schannel: failed to receive handshake, SSL/TLS connection failed); class=Net (12)
```

的错误上。面对频繁的`cargo check`，`cargo build`，`cargo run`命令执行和各种莫名其妙的“全量·索引同步”，“搬梯子”的经济成本（特别是“按流量·计费”）对个人来讲有点些高了 —— 那走的不是流量，而是我的“腰子”。

### 最理想的使用模型

* 仅**首次**编译或**依赖项变更**时，`cargo`命令才【连线】编译与同步本地的[crates.io-index](https://github.com/rust-lang/crates.io-index)索引清单 —— 有限且可控的“搬梯子”还是可以经济承受的。
* 在所有其它时候，`cargo`命令皆【离线】编译 —— 没事少连线[github.com](https://github.com)。

## 工作原理

`cargo-offline`命令会

1. 透传所有命令行参数给底层的`cargo`指令
2. 寻找距离`cargo-offline`指令执行目录最近的`Cargo.toml`文件，无论该配置文件
   1. 是【工作区`workspace`】配置文件
   2. 还是【工作区·成员`workspace.member`】配置文件。
3. 比较被找到`Cargo.toml`文件近期是否被修改过 —— 就是对比文件的【最后修改时间】属性值是否发生了变化。
4. 若`Cargo.toml`文件的·最后修改时间·变化了，就在透传`cargo`命令行参数时，默默地添加一个`--offline`参数。
5. 于是，`cargo`命令就会进入【离线·编译】模式。

### `Cargo.toml`文件修改时间的保存位置

判断`Cargo.toml`文件是否被修改过，关键需要：

* 缓存最后一次编译时读取的`Cargo.toml`【文件修改时间】
* 再，使用其与当前【文件修改时间】比大小

而对`Cargo.toml`【文件修改时间】的保存位置，程序提供了两种选择：

* 直接保存到`Cargo.toml`文件自身里，和作为[metadata](https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table)配置块内一个键值对。
  * 就工作区的根工程而言，保存配置块是`[workspace.metadata]`
  * 就工作区的成员工程而言，保存配置块是`[package.metadata]`
  * 优点：
    * 不会在工程目录下引入新的文件了。
  * 缺点：
    * `toml crate`会改变`Cargo.toml`文件内原有的各个配置块的显示次序，更会把所有的“双引号”替换为“单引号”。
* 保存于一个独立的配置文件 —— 与`Cargo.toml`同级目录的`cargo-offline-config.toml`。
  * 优点：
    * `Cargo.toml`文件内的配置块不会被重新排序
  * 缺点：
    * 手工地向`.gitignore`文件添加`cargo-offline-config.toml`文件名。

此外，**这个【文件修改时间】保存位置的选择是【编译时·决策】，而不是【运行时·决策】。**即，以`Cargo features`作为编译条件，根据不同同的决策选择，将会编译输出不一样的二进行可执行文件。

## 安装

此命令行工具`crate`已经被发布至[crates.io](https://crates.io/)包仓库。所以，我就没有准备针对各个平台与架构的预编译包（交叉编译准备这些真心地太麻烦了。感谢伟大的包管理器）。相反，

想要缓存【`Cargo.toml`文件修改时间】至`Cargo.toml [metadata]`配置块的同学，执行这条指令安装：

```shell
cargo install --features=cargo-metadata
```

想要缓存【`Cargo.toml`文件修改时间】至`cargo-offline-config.toml`独立文件的同学，执行这条指令安装：

```shell
cargo install --features=toml-config
```

> 因为我没有给`Cargo Package`设置`default features`，所以完全忽略`--features=`命令行参数会导致源码编译错误。恶作剧地，同时指定`cargo-metadata`与`toml-config`自定义`cargo features`也会导致编译失败。

一旦被安装成功之后，`cargo-offline.exe`可执行文件就会

* 出现在`%CARGO_HOME%\bin`目录下
* 从`PATH`环境变量，可见
* 可从命令行直接运行

## 使用

`cargo-offline`命令的执行也有两种方式可供选择：

1. 独立执行`cargo-offline`。后随和标准`cargo`命令相同的命令行参数（这些参数会被透传给`cargo`指令的）。比如，

    ```shell
    cargo-offline check
    ```

2. 作为`cargo`指令的子命令来执行。也后随和标准`cargo`命令相同的命令行参数。比如，

    ```shell
    cargo offline check
    ```

## 源码也精彩，欢迎来品鉴

不仅是语句的堆叠，而是讲究了一点套路。包括但不限于：

1. 【条件编译】`plus`【策略·设计模式】 —— 解决`Cargo.toml`文件·修改时间·保存位置的选择问题。
   1. 【策略·模式】大约对等于`OOP`里的【控制反转`IoC`】`plus`【依赖注入`DI`】
   2. 若想深入了解【策略·模式】的细节理论，我推荐文章[浅聊Rust【策略·设计模式】Strategy / Policy design pattern](https://rustcc.cn/article?id=972a6d02-eee7-42c5-8cf6-a75cb8aa9cc6)。
2. `Builder`设计模式 —— 解决`struct`局部初始化的问题。
   1. 其大约对等于`OOP`里【工厂模式】的概念。
   2. 但，亲自给每个`struct`编写`Builder`不是傻吗！我的选择是[derive_builder](https://docs.rs/derive_builder)。
3. `Option / Result`枚举类的“拆箱/装箱”配合器【`Combinator`模式】 —— 避免丑陋且有`panic`风险的`.unwrap()`“拆箱”操作。
   1. 有那么一点儿`ramda`链式函数调用的感觉。馁馁的函数编程范式。
4. 规则宏`macro-by-example` —— 避免代码重复。
   1. 这是【结构相同·但·类型不同】代码块复用的利器呀！

### 关于·编译

**重要，十分重要**：因为【不稳定`feature`】`file_set_times`在程序中被·条件性·开启，所以整个`Cargo Package`工程的`rustup`工具链被锁定于`nightly`版本。若你`git clone`此工程至本地，请先安装`nightly`版的`rustc`再编译执行之。

我推荐使用`VSCode`来编辑与编译该`Cargo Package`工程

1. `Ctrl + Shift + B`直接编译+执行。
2. 在安装了`CodeLLDB`插件之后，`F5`就先编译，再进入断点调试模式了。

无论采用上面哪种方式编译程序，`VSCode`都会弹出【下拉·选择器】和要求你选择一条【自定义`cargo feature`】。请注意使用【上下箭头】与【回车】键，响应`VSCode`的选择要求。

## 后续路图

若今后给该·命令行工具·添加更多功能与配置选项，我计划上【`GUI`图形界面】，鉴于我之前对`win32`与`Gnome.GTK3`的编程经历与背景。
