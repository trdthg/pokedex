# 2021-08-26 - Rust 六边形架构 #3 - HTTP API

这篇文章是下面系列的一部分

- [Hexagonal architecture in Rust #1 - Domain](https://alexis-lozano.com/hexagonal-architecture-in-rust-1/)
- [Hexagonal architecture in Rust #2 - In-memory repository](https://alexis-lozano.com/hexagonal-architecture-in-rust-2/)
- [Hexagonal architecture in Rust #3 - HTTP API](https://alexis-lozano.com/hexagonal-architecture-in-rust-3/)
- [Hexagonal architecture in Rust #4 - Refactoring](https://alexis-lozano.com/hexagonal-architecture-in-rust-4/)
- [Hexagonal architecture in Rust #5 - Remaining use-cases](https://alexis-lozano.com/hexagonal-architecture-in-rust-5/)
- [Hexagonal architecture in Rust #6 - CLI](https://alexis-lozano.com/hexagonal-architecture-in-rust-6/)
- [Hexagonal architecture in Rust #7 - Long-lived repositories](https://alexis-lozano.com/hexagonal-architecture-in-rust-7/)

集结吧，我的战友们。今天我们将要战斗！谁？你问我。这些土地上不言而喻的恶魔：👿借用检查！

好了，让我们暂时停止这个指环王风格的印象，工作等待着我们 :)

在之前的文章中，我们定义了我们的 Domain 实体并实现了一个用例和一个存储库。

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

我们本可以把它交给我们的客户，但是除了运行测试之外，main.rs 文件仍然只输出一个 hello world。今天，我们将把我们的项目转换成一个返回 JSON
的 HTTP API。

## The HTTP API

如果你没记错的话，我没有在项目中使用过异步。这是为了专注于我们应用程序的架构。如果你真的想使用异步，那就去吧 :) 非异步 Web
框架并不多，但仍然有一些。我对本文的选择是 rouille，它将很好地处理我们的用例。

所以首先，我们打开 Cargo.toml 并将其添加到我们的依赖项中：

```toml
[dependencies]
rouille = "3.2.1"
```

现在让我们创建一个包含我们的 API 的文件夹。这里面包括 `mod.rs` 文件，我们将在其中添加基本的路由逻辑。我还将添加一个简单的 `health.rs`
文件来处理我们的第一个 route：

```
src
└── api
    ├── health.rs
    └── mod.rs
```

我们只会在 api 文件夹中使用到 rouille，如果在以后，我们想用 actix 代替 rouille，我们只需要修改 api
的部分即可(其实我们还要把一些函数转换为异步的，但是它与 Web 框架的选择是不相关的)

现在让我们添加一些代码，让基本的工作 API 在 `/health` 上发送 GET 请求时返回一些文本：

```rs
mod api;
mod domain;
mod repositories;

#[macro_use]
extern crate rouille;

fn main() {
    api::serve("localhost:8000");
}
```

接下来，在 `api/mod.rs` 里添加 `serve` 函数

```rs
mod health;

pub fn serve(url: &str) {
    rouille::start_server(url, move |req| {
        router!(req,
            (GET) (/health) => {
                health::serve()
            },
            _ => {
                rouille::Response::from(Status::NotFound)
            }
        )
    });
}
```

现在只需要编辑 `api/health.rs`：

```rs
use rouille;

pub fn serve() -> rouille::Response {
    rouille::Response::text("Gotta catch them all!")
}
```

现在您应该可以使用 `cargo run` 运行应用程序并使用浏览器访问
http://localhost:8000/health。在那里，一条美丽的信息在等着你

```
Gotta catch them all!
```

Great! But I told before we want a JSON API. Let's convert this endpoint to
return JSON then. serde will help us. rouille already uses some serde traits,
let's add the same version as rouille uses in our Cargo.toml. To do that I run
cargo tree | grep serde: 太棒了！但我之前说过我们想要一个 JSON API。让我们将这个API接口转换为返回 JSON。我们将用到
serde。rouille 已经使用了一些 serde 的 Trait，可以通过 `cargo tree | grep serde`查看：

```
├── serde v1.0.129
├── serde_derive v1.0.129 (proc-macro)
├── serde_json v1.0.66
│   └── serde v1.0.129
```

接着让我们在 Cargo.toml 中添加与 rouille 使用的版本相同的 serde 依赖。

```toml
[dependencies]
rouille = "3.2.1"
serde = { version = "1.0.129", features = ["derive"] }
serde_json = "1.0.66"
```

现在来编辑 `api/health.rs`:

```rs
use rouille;
use serde::Serialize;

#[derive(Serialize)]
struct Response {
    message: String,
}

pub fn serve() -> rouille::Response {
    rouille::Response::json(&Response {
        message: String::from("Gotta catch them all!"),
    })
}
```

在次访问你的浏览器 :tada: :D

```json
{
  "message": "Gotta catch them all!"
}
```

## 获取请求

_Fetch the request_

我们的客户想要的是能够创造一个宝可梦。首先，由于我们的 API 将是 RESTful，下面是我们将使用的 HTTP 请求的示例：

```
- POST http://localhost:8000
- Headers
    Content-Type: application/json
- Body
    {
        "number": 4,
        "name": "Charmander",
        "types": ["Fire"]
    }
```

现在，让我们回到 `api/mod.rs` 添加一个新的路由

```rs
mod create_pokemon;
mod health;

pub fn serve(url: &str) {
    rouille::start_server(url, move |req| {
        router!(req,
            ...
            (POST) (/) => {
                create_pokemon::serve(req)
            },
            ...
        )
    });
}
```

让我们创建一个新的文件 `api/create_pokemon.rs` 并写入下面的内容：

```rs
use rouille;
use serde::Serialize;

#[derive(Serialize)]
struct Response {
    message: String,
}

pub fn serve(_req: &rouille::Request) -> rouille::Response {
    rouille::Response::json(&Response {
        message: String::from("Pokemon created!"),
    })
}
```

现在您可以使用 REST 客户端 (postman、curl、...) 在 http://localhost:8000 上发送 POST 请求，body
可以是任何东西。你应该收到以下内容：

```json
{
  "message": "Pokemon created!"
}
```

但是当请求上下文不是我们想要的时，最好发回 400 状态码。让我们稍微编辑一下 `api/create_pokemon.rs`：

```rs
use crate::api::Status;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    number: u16,
    name: String,
    types: Vec<String>,
}

pub fn serve(req: &rouille::Request) -> rouille::Response {
    match rouille::input::json_input::<Request>(req) {
        Ok(_) => {}
        _ => return rouille::Response::from(Status::BadRequest),
    };
    ...
}
```

现在，如果发送一个没有 name 值的请求，或者如果 number 为负数，用户将会得到 400 状态码。

## 添加存储库

_Plug the repository_

好的，但是宝可梦实际上既没有创建也没有添加到存储库中。而且也不会调用Usecase！让我们首先在 `main.rs` 中添加一个内存存储库：

```rs
use repositories::pokemon::InMemoryRepository;

fn main() {
    let repo = InMemoryRepository::new();
    api::serve("localhost:8000", &mut repo);
}
```

现在，我们必须相应地编辑 `api/mod.rs`：

```rs
use crate::repositories::pokemon::Repository;

pub fn serve(url: &str, repo: &mut dyn Repository) {
    rouille::start_server(url, move |req| {
        router!(req,
            ...
            (POST) (/) => {
                create_pokemon::serve(repo, req)
            },
            ...
        )
    });
}
```

别忘了修改 `api/create_pokemon.rs`：

```rs
use crate::repositories::pokemon::Repository;

pub fn serve(_repo: &mut dyn Repository, req: &rouille::Request) -> rouille::Response {
```

你现在可以运行 cargo run 了，它应该 ...

```
error[E0277]: `dyn Repository` cannot be sent between threads safely
= help: the trait `Send` is not implemented for `dyn Repository`
error[E0277]: `dyn Repository` cannot be shared between threads safely
= help: the trait `Sync` is not implemented for `dyn Repository`
error: aborting due to 2 previous errors
```

我只保留了最基础的错误日志。有些东西不起作用，这是因为......借用检查器。我的意思是这其实是我的错，但是借用检查器在罩着我们 :)

## 打败借用检查器

_Checkmate for the borrow-checker_

像往常一样，编译器很有帮助：它告诉我们在 `Repository` 上实现 `Send` 和 `Sync`。让我们修改
`repositories/pokemon.rs` 来实现这一点：

```rs
pub trait Repository: Send + Sync {
    fn insert(&mut self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert;
}
```

Rust 很容易，对吧？我们的修复将非常快，因为一旦 `cargo run`：

```
error[E0621]: explicit lifetime required in the type of `repo`
 --> src/api/mod.rs:7:5
  |
6 | pub fn serve(url: &str, repo: &mut dyn Repository) {
  |                               ------------------- help: add explicit lifetime `'static` to the type of `repo`: `&'static mut (dyn Repository + 'static)`
```

现在，编译器告诉我们存储库上需要一个“静态生命周期”。让我们考虑一下。这里真正的问题是什么？我们希望将存储库的引用发送到为每个请求生成的线程中。现在我们使用我们的
InMemory 结构创建了一个存储库。问题是，当我们的应用程序执行到主函数结束时，InMemory 将被丢弃。
但也许有些线程仍然会引用这个结构。因此可能会导致编译器错误。

但是，我们想要的是以某种方式告诉程序仅在引用不再存在时再删除 InMemory。这种方式称为引用计数器。我们很幸运，Rust
为此提供了两种类型，其中一种是专门为在线程之间安全共享而创建的。它的名字是 **Arc**，这就是我们将要使用的。

因此，让我们在 `main.rs` 中用 Arc 包装我们的存储库：

```rs
use std::sync::Arc;

fn main() {
    let repo = Arc::new(InMemoryRepository::new());
    api::serve("localhost:8000", repo);
}
```

你可以看到我们移除了两个东西：一个 `&` 和一个 `mut`。 Arc
实际上是一个指针，因此它的大小在编译时是已知的。它指向位于堆中的存储库。因此我们不需要引用它。其次，Arc
是不可变的，所以我们必须使用内部可变性。我们稍后再谈。

现在让我们修改 api/mod.rs：

```rs
use std::sync::Arc;

pub fn serve(url: &str, repo: Arc<dyn Repository>) {
    rouille::start_server(url, move |req| {
        router!(req,
            ...
            (POST) (/) => {
                create_pokemon::serve(repo.clone(), req)
            },
            ...
        )
    });
}
```

最后再来修改 `api/create_pokemon.rs:`

```rs
use std::sync::Arc;

pub fn serve(_repo: Arc<dyn Repository>, req: &rouille::Request) -> rouille::Response {
```

编译成功 \o/

## Domain 也需要爱 💓

我们围绕着一个 Domain 设计了我们的应用程序，其中包含使用 Usecase 获取数据和一个存储库。像以前一样，我们也必须在用例中把存储库替换对 Arc
的可变引用。好在我现在只实现了一个用例 :) 让我们在 `domain/create_pokemon.rs` 中修改函数签名：

```rs
use std::sync::Arc;

fn execute(repo: Arc<dyn Repository>, req: Request) -> Response {
```

不要忘记测试！

```rs
let repo = Arc::new(InMemoryRepository::new());
let res = execute(repo, req);
```

在 cargo run 中，我们偶然发现了我之前讨论过的一个问题：Arc 是不可变的。

```
25 |         (Ok(number), Ok(name), Ok(types)) => match repo.insert(number, name, types) {
   |                                                    ^^^^ cannot borrow as mutable
```

如果我们检查 `repositories/pokemon.rs` 中的 `Repository`，我们实际上可以看到 `insert` 函数期望存储库是可变的：

```rs
pub trait Repository: Send + Sync {
    fn insert(&mut self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert;
}
```

所以我们将在 trait 和我们的实现中删除这个 mut :) 让我们运行 cargo run：

```rs
36 |     fn insert(&self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert {
   |               ----- help: consider changing this to be a mutable reference: `&mut self`
...
46 |         self.pokemons.push(Pokemon::new(number_clone, name, types));
   |         ^^^^^^^^^^^^^ `self` is a `&` reference, so the data it refers to cannot be borrowed as mutable
```

哎呀，这个错误信息不是很有帮助。我们刚刚删除了 mut，现在编译器希望我们重新添加它。实际上这是合乎逻辑的，编译器不知道存储库在 Arc 中。

有趣的是，问题不再在于 trait，而在于我们的 InMemory 实现。我们需要能够在 self 不可变的情况下改变 pokemon。 这就是内部可变性。
而且，Rust 再次为此提供了一些原语！ 我们将选择 Mutex 原语，因为它是为了在线程之间共享数据而设计的。因此，让我们将 pokemon 包装到
Mutex 中：

```rs
use std::sync::Mutex;

pub struct InMemoryRepository {
    error: bool,
    pokemons: Mutex<Vec<Pokemon>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        let pokemons: Mutex<Vec<Pokemon>> = Mutex::new(vec![]);
        Self {
            error: false,
            pokemons,
        }
    }
}
```

现在，我们必须锁定 Mutex 才能读取或写入宝可梦。锁定 Mutex 意味着线程轮流等待读取或写入它所保存的数据，因此始终只有一个线程访问数据。

```rs
impl Repository for InMemoryRepository {
    fn insert(&self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert {
        if self.error {
            return Insert::Error;
        }

        let mut lock = match self.pokemons.lock() {
            Ok(lock) => lock,
            _ => return Insert::Error,
        };

        if lock.iter().any(|pokemon| pokemon.number == number) {
            return Insert::Conflict;
        }

        let number_clone = number.clone();
        lock.push(Pokemon::new(number_clone, name, types));
        Insert::Ok(number)
    }
}
```

现在它编译通过，并且所有的测试也仍然通过！

## API + domain = <3

是时候将 API 连接到 Domain 了。让我们编辑 `api/create_pokemon.rs`：

```rs
use crate::domain::create_pokemon;

pub fn serve(repo: Arc<dyn Repository>, req: &rouille::Request) -> rouille::Response {
    let req = match rouille::input::json_input::<Request>(req) {
        Ok(req) => create_pokemon::Request {
            number: req.number,
            name: req.name,
            types: req.types,
        },
        _ => return rouille::Response::from(Status::BadRequest),
    };
    match create_pokemon::execute(repo, req) {
        create_pokemon::Response::Ok(number) => rouille::Response::json(&Response { number }),
        create_pokemon::Response::BadRequest => rouille::Response::from(Status::BadRequest),
        create_pokemon::Response::Conflict => rouille::Response::from(Status::Conflict),
        create_pokemon::Response::Error => rouille::Response::from(Status::InternalServerError),
    }
}
```

记得把 Domain 中需要的改为 pub：

```rs
// domain/mod.rs
pub mod create_pokemon;

// domain/create_pokemon.rs
pub struct Request {
    pub number: u16,
    pub name: String,
    pub types: Vec<String>,
}

pub enum Response {
    ...
}

pub fn execute(repo: Arc<dyn Repository>, req: Request) -> Response {
    ...
}
```

在次运行 cargo run 并向 create_pokemon 路由发送有效请求：

```json
{
  "number": 30
}
```

\o/

## 下一步

_Next steps_

这篇文章比预期的要长，对此我感到抱歉。希望它对你有用 :) 在下一篇文章中，我将实现其他 Usecase (客户厌倦了等待我解释一切，客户真糟糕 :p)。
之后，我将实现其他的前端和存储库，以更好地了解六边形架构的强大功能。如果您想收到下一篇文章发布的通知，请随时订阅 rss 提要或我的推特。

像往常一样，代码可以在 [github](https://github.com/alexislozano/pokedex/tree/article-3)
上访问。
