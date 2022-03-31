# 2021-08-24 - Rust 六边形架构 #2 - In-memory repository

这篇文章是下面系列的一部分

- [Hexagonal architecture in Rust #1 - Domain](https://alexis-lozano.com/hexagonal-architecture-in-rust-1/)
- [Hexagonal architecture in Rust #2 - In-memory repository](https://alexis-lozano.com/hexagonal-architecture-in-rust-2/)
- [Hexagonal architecture in Rust #3 - HTTP API](https://alexis-lozano.com/hexagonal-architecture-in-rust-3/)
- [Hexagonal architecture in Rust #4 - Refactoring](https://alexis-lozano.com/hexagonal-architecture-in-rust-4/)
- [Hexagonal architecture in Rust #5 - Remaining use-cases](https://alexis-lozano.com/hexagonal-architecture-in-rust-5/)
- [Hexagonal architecture in Rust #6 - CLI](https://alexis-lozano.com/hexagonal-architecture-in-rust-6/)
- [Hexagonal architecture in Rust #7 - Long-lived repositories](https://alexis-lozano.com/hexagonal-architecture-in-rust-7/)

> 免责声明：在本文中，我将对存储库使用一个简单的可变引用。因为现在我们只是在测试中使用它。我将在下一篇文章中进行更改 :)

在上一篇文章中，我已经开始创建基本的项目和架构。我们已经有了一个带有一个用例和一些实体的`Domain`模块：

```
src
├── domain
│   ├── create_pokemon.rs
│   ├── entities.rs
│   └── mod.rs
└── main.rs
```

## 内存中的数据库

_In-memory repository_

让我们回到我们的 create_pokemon 用例。
目前，它可以在成功时返回宝可梦的数量，当请求不符合业务规则时会返回一个错误。但现在并没有实际存储宝可梦的地方。让我们来解决这个问题！现在你应该知道我喜欢从什么开始：一个测试
:)。这个测试将检查我们不能有两个相同id的宝可梦。

```rs
use crate::repositories::pokemon::InMemoryRepository;

#[test]
fn it_should_return_a_conflict_error_when_pokemon_number_already_exists() {
    let number = PokemonNumber::try_from(25).unwrap();
    let name = PokemonName::try_from(String::from("Pikachu")).unwrap();
    let types = PokemonTypes::try_from(vec![String::from("Electric")]).unwrap();
    let mut repo = InMemoryRepository::new();
    repo.insert(number, name, types);
    let req = Request {
        number: 25,
        name: String::from("Charmander"),
        types: vec![String::from("Fire")],
    };

    let res = execute(&mut repo, req);

    match res {
        Response::Conflict => {}
        _ => unreachable!(),
    }
}
```

在个测试用例里，我们直接在存储库中插入一个 Pokemon。然后我们尝试使用 Usecase 再次插入具有相同编号的宝可梦。Usecase
应该返回一个冲突错误。

像之前一样，它不能通过编译，因为这里的很多代码都不存在。让我们首先将 `Conflict` 错误添加到 `Response` 中：

```rs
enum Response {
    ...
    Conflict,
}
```

接着，在宝可梦类型中填加一个火属性

```rs
enum PokemonType {
    Electric,
    Fire,
}

impl TryFrom<String> for PokemonType {
    type Error = ();

    fn try_from(t: String) -> Result<Self, Self::Error> {
        match t.as_str() {
            "Electric" => Ok(Self::Electric),
            "Fire" => Ok(Self::Fire),
            _ => Err(()),
        }
    }
}
```

您可能想知道 InMemoryRepository 是什么。 它在我们在不知道 Reposity
具体选择那个数据库时使用的数据库，这是我们的第一个实现，主要用于测试。因为它可以像真正的 Reposity
一样工作，所以我们将能够使用它去向客户展示我们的进度并要求他提供反馈。现在让我们修改 `Usecase`，并向其中添加
`repo: &mut dyn Reposity` 参数。

```rs
use crate::repositories::pokemon::Repository;

fn execute(repo: &mut dyn Repository, req: Request) -> Response {
```

这里需要注意的一点是，`execute` 函数并不会得到具体的 `Reposity` 实现，而是任何实现了 `Reposity` Trait
的结构体。让我们在之前的两个测试用例中也补充 repo 参数：

```rs
#[test]
fn it_should_return_a_bad_request_error_when_request_is_invalid() {
    let mut repo = InMemoryRepository::new();
    let req = Request {
    ...
    let res = execute(&mut repo, req);
    ...
}

#[test]
fn it_should_return_the_pokemon_number_otherwise() {
    let mut repo = InMemoryRepository::new();
    let number = 25;
    ...
    let res = execute(&mut repo, req);
    ...
}
```

接下来，我们将在新模块`repositories/pokemon.rs`中去定义 `InMemoryRepository` 和 `Repository`

```
src
├── domain
│   ├── create_pokemon.rs
│   ├── entities.rs
│   └── mod.rs
├── repositories
│   ├── mod.rs
│   └── pokemon.rs
└── main.rs
```

不要忘了添加模块

```rs
// main.rs
mod repositories;

// repositories/mod.rs
pub mod pokemon;
```

```rs
pub trait Repository {}

pub struct InMemoryRepository;

impl Repository for InMemoryRepository {}
```

接下来，让我们实现 `InMemoryRepository` 的 `new` 方法。在这里，InMemoryRepository 只是简单的存储了一个
`Pokemon` 列表

```rs
use crate::domain::entities::Pokemon;

pub struct InMemoryRepository {
    pokemons: Vec<Pokemon>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        let pokemons: Vec<Pokemon> = vec![];
        Self { pokemons }
    }
}
```

现在，终于是实现 `Pokemon` 实体的时候了

```rs
pub struct Pokemon {
    pub number: PokemonNumber,
    name: PokemonName,
    types: PokemonTypes,
}

impl Pokemon {
    pub fn new(number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Self {
        Self {
            number,
            name,
            types
        }
    }
}
```

同时，我们需要将 `entities.rs` 转为 `pub`

```rs
// domain/mod.rs
pub mod entities;
```

现在唯一没有被实现的就剩下 `insert` 方法了，因为我们希望能够在任何实现了 `Repository Trait` 的 repository
上都能调用该方法，所以需要在 Trait 上添加一个函数签名, 并为 InMemoryRepository 实现 `insert` 方法：

```rs
use crate::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};

pub trait Repository {
    fn insert(&self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> PokemonNumber;
}

impl Repository for InMemoryRepository {
    fn insert(&self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> PokemonNumber {
        number
    }
}
```

让我们尝试运行测试：

```
cargo test
running 3 tests
test it_should_return_a_bad_request_error_when_request_is_invalid ... ok
test it_should_return_the_pokemon_number_otherwise ... ok
test it_should_return_a_conflict_error_when_pokemon_number_already_exists ... FAILED
```

第三个测试失败了，插入成功时，insert 应该返回一个宝可梦的编号，如果对应的编号已经存在，需要返回一个冲突的错误。所以我们现在要填加一个 `Insert`
结构体表示这两种结果，同时要将原本的不可边借用变为可变借用：

```rs
pub enum Insert {
    Ok(PokemonNumber),
    Conflict,
}

pub trait Repository {
    fn insert(&mut self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert;
}
```

接下来实现 InMemoryRepository 的 insert 方法：

```rs
impl Repository for InMemoryRepository {
    fn insert(&mut self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert {
        if self.pokemons.iter().any(|pokemon| pokemon.number == number) {
            return Insert::Conflict;
        }

        let number_clone = number.clone();
        self.pokemons.push(Pokemon::new(number_clone, name, types));
        Insert::Ok(number)
    }
}
```

为了让 `clone` 和 `==`，我们还要为 `PokemonNumber` 实现 `PartialEq` 和 `Clone` 两个 Trait：

```rs
use std::cmp::PartialEq;

#[derive(PartialEq, Clone)]
pub struct PokemonNumber(u16);
```

最后，让 `execute` 函数调用 insert 方法

```rs
fn execute(repo: &mut dyn Repository, req: Request) -> Response {
    match (
        PokemonNumber::try_from(req.number),
        PokemonName::try_from(req.name),
        PokemonTypes::try_from(req.types),
    ) {
        (Ok(number), Ok(name), Ok(types)) => match repo.insert(number, name, types) {
            Insert::Ok(number) => Response::Ok(u16::from(number)),
            Insert::Conflict => Response::Conflict,
        },
        _ => Response::BadRequest,
    }
}
```

再次运行测试

```
cargo test
running 3 tests
test it_should_return_a_bad_request_error_when_request_is_invalid ... ok
test it_should_return_the_pokemon_number_otherwise ... ok
test it_should_return_a_conflict_error_when_pokemon_number_already_exists ... ok
```

太棒了，冲突测试也通过了！

## 以为已经结束了吗？

_You thought we were done?_

没有。在存储库中还有一种问题会发生。假设由于某些意外，存储库无法正常工作。如果是数据库，那就是连接错误，如果是 API，那就是网络错误。我们也应该处理这种情况。

你知道我现在要做什么：写一个测试！

```rs
#[test]
fn it_should_return_an_error_when_an_unexpected_error_happens() {
    let mut repo = InMemoryRepository::new().with_error();
    let number = 25;
    let req = Request {
        number,
        name: String::from("Pikachu"),
        types: vec![String::from("Electric")],
    };

    let res = execute(&mut repo, req);

    match res {
        Response::Error => {}
        _ => unreachable!(),
    };
}
```

这个测试有两个不同点。第一，我们添加了 `with_error` 方法表示存储库连接异常。第二，我们需要检查 Respnse 是否发生异常。

首先为 Response 添加一个新的类型

```rs
enum Response {
    ...
    Error,
}
```

现在我们要实现 `with_error` 方法，我的想法是在 `InMemoryRepository` 中填加一个 `error`
字段，表示是否会在连接存储库是进行检查。如果 `error = true` 我们就返回一个 error，否则返回正常结果：

```rs
pub enum Insert {
    ...
    Error,
}

pub struct InMemoryRepository {
    error: bool,
    pokemons: Vec<Pokemon>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        let pokemons: Vec<Pokemon> = vec![];
        Self {
            error: false,
            pokemons,
        }
    }

    pub fn with_error(self) -> Self {
        Self {
            error: true,
            ..self
        }
    }
}

impl Repository for InMemoryRepository {
    fn insert(&mut self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert {
        if self.error {
            return Insert::Error;
        }

        if self.pokemons.iter().any(|pokemon| pokemon.number == number) {
            return Insert::Conflict;
        }

        let number_clone = number.clone();
        self.pokemons.push(Pokemon::new(number_clone, name, types));
        Insert::Ok(number)
    }
}
```

同样的，在 `execute` 中处理这种情况

```rs
fn execute(repo: &mut dyn Repository, req: Request) -> Response {
    match (
        PokemonNumber::try_from(req.number),
        PokemonName::try_from(req.name),
        PokemonTypes::try_from(req.types),
    ) {
        (Ok(number), Ok(name), Ok(types)) => match repo.insert(number, name, types) {
            Insert::Ok(number) => Response::Ok(u16::from(number)),
            Insert::Conflict => Response::Conflict,
            Insert::Error => Response::Error,
        },
        _ => Response::BadRequest,
    }
}
```

让我们运行测试

```
cargo test
running 4 tests
test it_should_return_a_bad_request_error_when_request_is_invalid ... ok
test it_should_return_a_conflict_error_when_pokemon_number_already_exists ... ok
test it_should_return_an_error_when_an_unexpected_error_happens ... ok
test it_should_return_the_pokemon_number_otherwise ... ok
```

\o/

## 下一步

这篇文章的长度已经足够了，让我们暂听到这里。下一次，我会将前端部分实现为 HTTP API。之后我会处理其他的 use
cases。我还会实现更多的存储库和前端，这些功能会通过命令行参数进行开启。

和以前一样，我会在 [github](https://github.com/alexislozano/pokedex/tree/article-2)
上创建一个包含所有更改的分支。
