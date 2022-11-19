# `cargo-offline`命令

`cargo-offline`是标准`cargo`命令的包装器。其被用来，根据·距离`cargo-offline`命令执行目录最近的`Cargo.toml`文件是否被修改过，来给被包装的`cargo`命令条件地增补`--offline`命令行参数（即，离线编译）。形象地讲，就是将`cargo check`条件地变形为`cargo check --offline`。

## 动机

最近一段时间，[github.com](https://github.com)访问的稳定性实在**很差**。但，执行`cargo`命令总是要求

* 首先，同步[crates.io-index](https://github.com/rust-lang/crates.io-index)索引清单。
* 然后，执行目标任务

于是，日常开发/编译工作流就时常被阻塞于

```shell
warning: spurious network error (1 tries remaining): [35] SSL connect error (schannel: failed to receive handshake, SSL/TLS connection failed); class=Net (12)
Caused by:
  Unable to update registry `crates-io`
Caused by:
  failed to fetch `https://github.com/rust-lang/crates.io-index`
Caused by:
  [35] SSL connect error (schannel: failed to receive handshake, SSL/TLS connection failed); class=Net (12)
```

的网络错误上。这实在令人感觉挫败！

另一方面，虽然“搬梯子”能够缓解问题，但面对频繁的`cargo check/run`指令执行（特别是，莫名其妙出现的“**全量**索引同步”现象），其“按流量·计费”的经济成本着实令人肉疼。

所以，我下定决心在业余时间搞一个【条件·离线·编译】的命令行工具，来拯救自己于迷茫。

### 最理想的使用模型

* 仅**首次**编译·或·在**依赖项变更**时，`cargo`命令才【连线】编译与同步本地的[crates.io-index](https://github.com/rust-lang/crates.io-index)索引清单 —— 有限且可控的“搬梯子”还是可以经济承受的。
* 在所有其它时候，`cargo`命令皆【离线】编译 —— 没事少连线[github.com](https://github.com)。

## 工作原理

`cargo-offline`命令会

1. 透传所有命令行参数给底层的`cargo`指令
2. 寻找距离`cargo-offline`执行目录最近的`Cargo.toml`文件，无论该配置文件
   1. 是【工作区`workspace`】配置文件
   2. 还是【工作区·成员`workspace.member`】配置文件。
3. 比较被找到的`Cargo.toml`文件·是否·被修改过 —— 就是对比该文件的【最后·修改时间】属性值是否发生了变化。
4. 若`Cargo.toml`文件的·最后修改时间·变化了，就给被透传的参数列表额外添加`--offline`参数项。
5. 于是，`cargo`命令就会进入【离线模式】编译了。

### `Cargo.toml`文件修改时间的保存位置

判断`Cargo.toml`文件·是否·被修改过，关键需要：

* 缓存·在上一次编译时·读取的`Cargo.toml`文件【修改时间】属性值
* 再，使用【缓存·时间值】与当前【文件修改时间】比大小

就将`Cargo.toml`文件【修改时间】保存于何处，`cargo-offline`程序提供了两套备选方案：

* 直接保存到`Cargo.toml`文件自身里，和作为[***.metadata](https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table)配置块内一个键值对。
  * 就【工作区】而言，保存配置块是`[workspace.metadata]`
  * 就【工作区·成员】和【普通工程】而言，保存配置块是`[package.metadata]`
  * 优点：
    * 不会在工程目录下引入新文件了。
    * 也不用修改`.gitignore`文件添加例外规则了。
  * 缺点：
    * 被`toml crate`编辑过的`Cargo.toml`文件，它内部
      * “配置块”会被重新排序
      * “双引号”会被替换为“单引号”。
    * 程序·会额外地依赖`cargo_toml crate`。所以，编译输出的二进制文件会更大那么一点点儿。
    * 编译指令·会额外地开启【不稳定`feature`】`file_set_times`
* 保存于独立的`*.toml`配置文件内。
  * 即，与`Cargo.toml`文件同目录的`cargo-offline-config.toml`文件。目前，此文件名是在代码内被硬编码的。
  * 优点：
    * `Cargo.toml`文件可保持“无损”。
    * 少一个程序依赖项
    * 避免开启【不稳定`feature`】
  * 缺点：
    * 需手工地向`.gitignore`文件添加`cargo-offline-config.toml`文件名。

值得一提的是，**`Cargo.toml`文件【修改时间】保存位置的选择是【编译时·决策】，而不是【运行时·决策】。**即，

* 以`Cargo features`作为编译条件
* 根据不同的决策选择
* 编译输出**不一样的**二进制行可执行文件作为结果。

## 安装

此命令行工具`crate`已经被发布至[crates.io](https://crates.io/)包仓库。所以，我就未对各主流平台与架构准备·预编译包（感谢伟大的包管理器！）。

* 选择缓存`Cargo.toml`文件【修改时间】至`Cargo.toml [metadata]`的同学，执行这条安装指令：

    ```shell
    cargo install --features=cargo-metadata
    ```

* 选择缓存`Cargo.toml`文件【修改时间】至`cargo-offline-config.toml`独立文件的同学，执行这条安装指令：

    ```shell
    cargo install --features=toml-config
    ```

因为我没有给`Cargo Package`设置`default features`，所以完全忽略`--features=`命令行参数会导致源码编译错误。恶作剧地，同时指定`--features=cargo-metadata`与`--features=toml-config`也会导致编译失败。

一旦被安装成功之后，`cargo-offline.exe`可执行文件就会

* 出现在`%CARGO_HOME%\bin`目录下
* 从`PATH`环境变量划定的搜索范围，可见
* 可从命令行直接运行

## 使用

`cargo-offline`命令的执行也有两种方式可供选择：

1. 作为独立命令，执行`cargo-offline`。后随和标准`cargo`命令相同的命令行参数（这些参数会被透传给`cargo`指令的）。比如，

    ```shell
    cargo-offline check
    ```

2. 作为`cargo`指令的子命令，执行`cargo offline`。比如，

    ```shell
    cargo offline check
    ```

`cargo-offline`的命令行参数与`cargo`完全相同，因为`cargo-offline`仅只做了透传处理。

## 源码也精彩，欢迎来品鉴

不是语句的堆叠，而是讲究了“套路”。被涉及到的【设计模式】包括但不限于：

1. 【条件编译】`plus`【策略·设计模式】 —— 解决`Cargo.toml`文件【修改时间】保存位置的选择问题。
   1. 【策略·模式】大约对等于`OOP`里的【控制反转`IoC`】`plus`【依赖注入`DI`】的组合。在我的代码，从`IoC`容器到`DI`注入项都是自写的。
   2. 欲深入了解【策略·模式】的细节理论，我推荐文章[浅聊Rust【策略·设计模式】Strategy / Policy design pattern](https://rustcc.cn/article?id=972a6d02-eee7-42c5-8cf6-a75cb8aa9cc6) —— 欢迎点赞、发评论与转发分享。
2. `Builder`设计模式 —— 解决`struct`局部初始化的问题。
   1. 其大约对等于`OOP`里【工厂模式】。
   2. 但，亲手给每个`struct`编写`Builder`，那不是傻吗！多大的工作量呀！我的选择是[derive_builder](https://docs.rs/derive_builder)。
3. `Option / Result`枚举类的“拆/装箱”配合器【`Combinator`模式】 —— 避免丑陋且有`panic`风险的`.unwrap()`“拆箱”操作。
   1. 有那么一点儿`ramda`链式函数调用的感觉了。馁馁的【函数编程·范式】。
4. 规则宏`macro-by-example` —— 避免代码重复。
   1. 这是【结构相同·但·类型不同】代码块复用的利器呀！
   2. 以【宏】的思维来复用代码，得花费一段时间来适用。

### 关于·编译

**重要，十分重要**：因为【不稳定`feature`】`file_set_times`在程序中被**条件地**开启，所以该`Cargo Package`工程依赖的`rustup`工具链被鲜明地**锁定于**`nightly`版本。若你`git clone`此工程至本地，请先安装`nightly`版的`rustc`再编译执行之。否则，会报错的。

另外，推荐使用`VSCode`编辑与编译`cargo-offline`工程，因为我已经配置好了：

1. `Ctrl + Shift + B`直接·编译`+`执行。
2. 在安装了`CodeLLDB`插件之后，`F5`就先编译，再进入断点调试模式。

无论采用上面哪种方式编译程序，`VSCode`都会弹出【下拉·选择器】，要求选择输入【自定义`cargo feature`】。所以，请注意使用【上下箭头】与【回车】键，响应`VSCode`的选择要求。

## 后续路图

若今后给该·命令行工具·添加更多功能与配置选项，我计划上【`GUI`图形界面】，考虑到我的`win32`与`Gnome.GTK3`编程经历与背景。
