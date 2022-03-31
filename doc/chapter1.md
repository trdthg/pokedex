# 2021-08-21 - Rust 六边形架构 #1 Domain

这篇文章是下面系列的一部分

- [Hexagonal architecture in Rust #1 - Domain](https://alexis-lozano.com/hexagonal-architecture-in-rust-1/)
- [Hexagonal architecture in Rust #2 - In-memory repository](https://alexis-lozano.com/hexagonal-architecture-in-rust-2/)
- [Hexagonal architecture in Rust #3 - HTTP API](https://alexis-lozano.com/hexagonal-architecture-in-rust-3/)
- [Hexagonal architecture in Rust #4 - Refactoring](https://alexis-lozano.com/hexagonal-architecture-in-rust-4/)
- [Hexagonal architecture in Rust #5 - Remaining use-cases](https://alexis-lozano.com/hexagonal-architecture-in-rust-5/)
- [Hexagonal architecture in Rust #6 - CLI](https://alexis-lozano.com/hexagonal-architecture-in-rust-6/)
- [Hexagonal architecture in Rust #7 - Long-lived repositories](https://alexis-lozano.com/hexagonal-architecture-in-rust-7/)

一段时间以来，我一直在阅读很多关于六边形架构、干净架构等的文章和书籍。我也看过了很多演讲。在学习这些主题的这段时间里，我一直在想如何在 Rust
中实现它们，因为我知道所有权模型可能会让它变得困难。

这篇文章可能会是我用来展示如何使用我提到的模式来实现软件的系列文章的第一篇。

## 六边形架构

_Hexagonal architecture_

六边形架构、洋葱架构、干净架构……这些架构其实都是一回事，所以从现在开始我会主要介绍六边形架构。

这个想法(六边形架构)的是让你的应用程序(application)的核心部分独立于它的依赖项(dependencies)。核心部分通常称为**域(Domain)**，它是所有业务规则(business)和实体(entity)所在的位置。依赖项基本上是应用程序的其余部分：数据库、框架、库、消息队列等等都包含在内。从本质上讲，这种架构是一种将业务部分与实现细节解耦的方法。

这种架构有以下一些优点：

- 你可以更改 Domain 而不更改依赖
- 你可以在不更改 Dmain 的情况下更改依赖
- 你可以更容易测试 Domain
- 你可以在需要时考虑使用哪些依赖，而不是在一开始就去实现细节

这种架构有几个优点：

## 一个疯狂的业务需求出现了！

_A wild business need appears!_

一个早上，我们的客户来找我们，我们开始以下对话：

- 嗨，我需要一个软件来管理宝可梦。
- 好的，你想对这些宝可梦做什么？
- 我需要创建新的宝可梦，删除它们，然后搜索它们。
- 大体了解了。您希望如何访问您的系统？使用浏览器还是使用终端？
- 呃，我真的不知道...
- 你想在哪里存放宝可梦？你们是否为我们提供对象存储服务的数据库或帐户？
- 什么是数据库？

在这里，您可以说客户不知道他想要什么。但事实上，目前我们真的不需要知道这些问题的答案。重要的是用例(Usecase)。让我们把客户的需求重写一下：

- 创建一只宝可梦
- 查询所有宝可梦
- 查询一只宝可梦
- 删除一只宝可梦

## 我们的第一个用例

_Our first use case_

我们的项目将用 Rust 实现，回收标题 :)， 让我们首先创建一个新的项目

```shell
cargo new pokedex
```

接着我们创建第一个用例:

```
src
├── domain
│   ├── create_pokemon.rs
│   └── mod.rs
└── main.rs
```

不要忘记加 `mod.rs`

```rs
// main.rs
mod domain;

// domain/mod.rs
mod create_pokemon;
```

我喜欢做的是首先编写测试，就好像代码已经编写好了一样。它帮助我创建一个干净的 API。所以我们可以打开 `domain/create_pokemon.rs`
并添加我们的第一个测试：

```rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_return_the_pokemon_number_otherwise() {
        let number = 25;
        let req = Request {
            number,
            name: String::from("Pikachu"),
            types: vec![String::from("Electric")],
        };

        let res = execute(req);

        assert_eq!(res, number);
    }
}
```

当然，现在还不能通过编译。首先我们需要创建一个 `Request` 结构体：

```rs
struct Request {
    number: u16,
    name: String,
    types: Vec<String>,
}
```

注意，我们没有在 `Request` 结构中使用花哨的类型。为什么？因为我们不希望调用我们用例的代码知道 **Domain**
中的实体。正如我之前所写，目标是拥有一个独立的 **Domain** 层。

现在，我们需要实现`execute`函数：

```rs
fn execute(req: Request) -> u16 {
    req.number
}
```

有用！让我们把它交给我们的客户！我不确定他拿到这个结果是否会高兴。实际上，我们还没有检查请求是否良好。如果 `number` 不在正确的范围内怎么办？如果给定的
`name` 是空字符串怎么办？如果宝可梦世界中不存在其中一种类型怎么办？让我们现在来解决这个问题 :)

## 实体

_Entities_

让我们添加一个新测试用例，用来检查用例在请求格式错误时会返回错误：

```rs
#[test]
fn it_should_return_a_bad_request_error_when_request_is_invalid() {
    let req = Request {
        number: 25,
        name: String::from(""),
        types: vec![String::from("Electric")],
    };

    let res = execute(req);

    match res {
        Response::BadRequest => {}
        _ => unreachable!(),
    };
}
```

因为没有实现 `Response`结构体, 所以现在还无法通过编译，现在用例(execute)只返回一个 `u16`，所以我们必须把它的返回结果改为
`Response`：

```rs
enum Response {
    Ok(u16),
    BadRequest,
}

fn execute(req: Request) -> Response {
    Response::BadRequest
}
```

我们还应该更改上一个测试用例去检查 `Ok` 情况：

```rs
match res {
    Response::Ok(res_number) => assert_eq!(res_number, number),
    _ => unreachable!(),
};
```

现在，代码编译成功了！ 但是检查 `Ok` 的测试失败了，因为现在 `execute` 只会返回 `Response::BadRequest`。
我们稍后会在来处理它。现在，我们要定义在请求中获得值的业务规则。让我们创建一个新文件 `domain/entities.rs` 来存储它们。

**宝可梦数量** _Pokemon number_

这个数字必须 `> 0`, `< 899`：

```rs
pub struct PokemonNumber(u16);

impl TryFrom<u16> for PokemonNumber {
    type Error = ();

    fn try_from(n: u16) -> Result<Self, Self::Error> {
        if n > 0 && n < 899 {
            Ok(Self(n))
        } else {
            Err(())
        }
    }
}

impl From<PokemonNumber> for u16 {
    fn from(n: PokemonNumber) -> u16 {
        n.0
    }
}
```

**宝可梦名称** _Pokemon name_

名字不能是空字符

```rs
pub struct PokemonName(String);

impl TryFrom<String> for PokemonName {
    type Error = ();

    fn try_from(n: String) -> Result<Self, Self::Error> {
        if n.is_empty() {
            Err(())
        } else {
            Ok(Self(n))
        }
    }
}
```

**宝可梦属性** _Pokemon types_

属性不能是空列表，而且所有类型都必须是已经定义过的。现在我们只定义一个电属性 `Electric`。

```rs
pub struct PokemonTypes(Vec<PokemonType>);

impl TryFrom<Vec<String>> for PokemonTypes {
    type Error = ();

    fn try_from(ts: Vec<String>) -> Result<Self, Self::Error> {
        if ts.is_empty() {
            Err(())
        } else {
            let mut pts = vec![];
            for t in ts.iter() {
                match PokemonType::try_from(String::from(t)) {
                    Ok(pt) => pts.push(pt),
                    _ => return Err(()),
                }
            }
            Ok(Self(pts))
        }
    }
}

enum PokemonType {
    Electric,
}

impl TryFrom<String> for PokemonType {
    type Error = ();

    fn try_from(t: String) -> Result<Self, Self::Error> {
        match t.as_str() {
            "Electric" => Ok(Self::Electric),
            _ => Err(()),
        }
    }
}
```

现在，我们去更新以下 `execute` 函数

```rs
fn execute(req: Request) -> Response {
    match (
        PokemonNumber::try_from(req.number),
        PokemonName::try_from(req.name),
        PokemonTypes::try_from(req.types),
    ) {
        (Ok(number), Ok(_), Ok(_)) => Response::Ok(u16::from(number)),
        _ => Response::BadRequest,
    }
}
```

干的好，所有测试都通过了！

## 下一步

_Next steps_

在下一篇文章中，我们将看到如何实现多个 **Reposity** 去存储宝可梦。所有的 Reposity 都会实现同一个
**`Trait`**，因此这些reposity能够非常方便的进行拓展(pluggable)和更换(exchangeable)，我们还将为
**Usecase** 给出多个实现，以便我们的系统能够适配多个前端 (interfaces)。
