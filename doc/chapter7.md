# 2021-10-17 - Rust 六边形架构 #7 - Long-lived repositories

It is now the time for the last article of the series. Thanks a lot to you,
anonymous reader who is still here after seven posts. We already have covered a
lot of things: we have a domain, we store data, and we can operate our program
with both a CLI and an HTTP server. So what do we still have to work on? Oh, I
see the client, let's ask him.
这里是本系列的最后一篇文章了。非常感谢你，在七篇文章之后仍然在这里的匿名读者。我们已经介绍了很多东西：我们有一个域，我们能够存储数据，我们可以使用 CLI 和
HTTP 服务器来操作我们的程序。那么我们还需要做什么呢？ 哦，我看到客户了，我们问问他吧。

- 嘿，最近好吗?
- 我很好，但你的软件不是很好.
- 哦... 有什么问题吗?
- 当我重启程序时, 所有的数据都丢失了.
- 啊，对，那很正常. 我们现在依然是在直接操作内存.
- 你能让这些数据得以长期保存吗？
- 这些数据？当然可以. 您能给我们提供保存的地方吗?
- 嗯，我不知道。我要去找人问问。你能不能让程序暂时使用硬盘存储？
- 马上完成！

好的，我们想要把数据保存在一个地方，当程序重启后也不会被删除。我们需要使用文件，同时因为我们想要使用一种可靠的方式去查询数据，我们将使用 SQL。文件和
SQL... 有东西在我脑海里尖叫者 **SQLite**

有一点好处是，我们现在已经实现了一个存储库系统，我们只需要填加一个新的存储库，以及一个命令行开关去决定使用那种存储库即可。因为有六边形架构，我们的 Domain
完全不用修改 :D

## 使用 SQLite 作为本地存储

### 创建数据库

首先我们要使用 sqlite3 提供的命令行工具创建一个数据库：

```
sqlite3 path/to/the/database.sqlite
```

你应该已经得到了 SQLite
的提示。现在我们要创建两张表。为什么是两张？因为一个宝可梦有很多种类型。所以我们不能再一列中存储所有的类型。但是实际上，宝可梦表与类型表是多对多的关系：

- 一只宝可梦有多种类型
- 一个类型可以对应多个宝可梦

所以完整的数据库应该是下面的样子，它一共有3张表：

```
pokemons          : | number          | name     |

types             : | id              | name     |

pokemons_to_types : | pokemons.number | types.id |
```

但是在这个例子里，我们实际上没有类型的 `id`，暂时我们只需要两张表：

```
pokemons: | number          | name |

types:    | pokemons.number | name |
```

让我们回到 SQLite 的命令行中。首先我们要激活外键(默认是关闭的)。

> 注意：这个操作只会在本次连接中生效，

```sql
pragma foreign_keys = 1;
```

现在我们开始创建数据表：

```sql
create table pokemons (
    number integer primary key,
    name text
);

create table types (
    pokemon_number integer,
    name text,
    foreign key (pokemon_number) references pokemons (number) on delete cascade,
    primary key (pokemon_number, name)
);
```

`on delete cascade` 的效果是当一个宝可梦被删除时，类型表中所有 pokemon_number 和 number
相同的行也会被删除。这样我们在程序中就不用在关心了:) 现在你可以使用 Ctrl-D 退出 SQLite 了。

## 添加选择存储库的开关

现在我们有 SQLite 作为数据库，我们要再添加一个命令行参数告诉程序我们想使用 SQLite。大概效果是这样
`pokedex --sqlite path/to/the/database.sqlite`。当你使用 cargo run
时需要改为这样：`cargo run -- --sqlite path/to/the/database.sqlite`。

```rs
fn main() { let repo = Arc::new(InMemoryRepository::new());

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("cli").long("cli").help("Runs in CLI mode"))
        .arg(Arg::with_name("sqlite").long("sqlite").value_name("PATH"))
        .get_matches();

    match matches.occurrences_of("cli") {
        0 => api::serve("localhost:8000", repo),
        _ => cli::run(repo),
    }
}
```

现在数据库开关已经加上了，和一通过 `--help` 查看：

```
OPTIONS: --sqlite <PATH>
```

接下来我们去初始化一个暂时还不存在的 `SqliteRepository`：

```rs
use repositories::pokemon::{..., SqliteRepository};

fn main() {
    let matches = ...

    let repo = build_repo(matches.value_of("sqlite"));

    match matches.occurrences_of("cli") {
        0 => api::serve("localhost:8000", repo),
        _ => cli::run(repo),
    }

}

fn build_repo(sqlite_value: Option<&str>) -> Arc<dyn Repository> {
    if let Some(path) = sqlite_value {
        match SqliteRepository::try_new(path) {
            Ok(repo) => return Arc::new(repo),
            _ => panic!("Error while creating sqlite repo"),
        }
    }

    Arc::new(InMemoryRepository::new())
}
```

当 `--sqlite` 打开时，程序会尝试初始化 SQLite 存储库。如果初始化失败，程序就会崩溃并打印错误信息。如果没有开启 `--sqlite`
就会使用默认的内存存储库。

## 实现 SQlite 存储库

我从文章一开始就在谈论 SQLite。现在我们需要一种从 Rust 调用数据库的方法。我们将使用 rusqlite 去实现。让我们在 Cargo.toml
中导入它：

```toml
[dependencies]
rusqlite = "0.26.0"
```

现在我们要实现 SQLiteRepository，再 InMemoryRepository 所在的文加中创建
SQLiteRepository。当然，这部是强制的，你也可以根据去要把它放在一个新文件中：

```rs
use rusqlite::Connection;

pub struct SqliteRepository { connection: Mutex<Connection>, }
```

main.rs 中的 use 现在应该不会报错了，接着实现 try_new 方法：

```rs
use rusqlite::{..., OpenFlags};

impl SqliteRepository {
    pub fn try_new(path: &str) -> Result<Self, ()> {
        let connection = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        {
            Ok(connection) => connection,
            _ => return Err(()),
        };

        match connection.execute("pragma foreign_keys = 1", []) {
            Ok(_) => Ok(Self {
                connection: Mutex::new(connection),
            }),
            _ => Err(()),
        }
    }

}
```

首先，我们调用 rusqlite 去新建一个对数据文件的连接。而且我们通过 OpenFlags 去明确禁止当文件不存在时 rusqulite
去自动创建数据文件。如果连接成功，我们就会执行只前提到的命令，确保外键开启。最后我们去返回这个存储库。

如果你现在尝试运行程序，还不能通过编译，为什么？因为我们还没有实现 Repository 定义的的所有方法：

```rs
impl Repository for SqliteRepository {
    fn insert(
        &self,
        number: PokemonNumber,
        name: PokemonName,
        types: PokemonTypes,
    ) -> Result<Pokemon, InsertError> {
        Err(InsertError::Unknown)
    }

    fn fetch_all(&self) -> Result<Vec<Pokemon>, FetchAllError>{
        Err(FetchAllError::Unknown)
    }

    fn fetch_one(&self, number: PokemonNumber) -> Result<Pokemon, FetchOneError> {
        Err(FetchOneError::Unknown)
    }

    fn delete(&self, number: PokemonNumber) -> Result<(), DeleteError> {
        Err(DeleteError::Unknown)
    }
}
```

编译通过了，但是我们的存储库依然没什么用现在。接下来我们要实现那些方法 :)

## 辅助函数

在我们实现存储库需要的函数之前，让我们先定义两个辅助函数。我们想要查询一个或者是所有宝可梦，用 SQL 语句描述就是一个
selectt。我们接下来要创建的辅助函数能够把查询逻辑整合到一起。

首先，我们编写一个能够拿到所有宝可梦编号和名字的函数。这个函数接受一个已经拿到锁的存储库，也可能会需要一个宝可梦的编号。如果函数接收到了编号，那么就会在 SQL
语句上添加 where 字句。这个函数会被添加到 `impl SqliteRepository`，因为它不是 `Repository Trait`
的一部分。另外，我们将直接使用原始数据( `u16` 和 `String` )，这个函数不应该是公开的。

```rs
use std::sync::{..., MutexGuard};

fn fetch_pokemon_rows(
    lock: &MutexGuard<'_, Connection>,
    number: Option<u16>,
) -> Result<Vec<(u16, String)>, ()> {
    // code will go here
}
```

好的，让我们开始吧。首先，我们要根据数字的存在来定义查询语句和查询参数：

```rs
let (query, params) = match number {
    Some(number) => (
        "select number, name from pokemons where number = ?",
        vec![number],
    ),
    _ => ("select number, name from pokemons", vec![]),
};
```

相当简单吧？现在我们必须准备一个 `statment` 并传递我们的参数：

```rs
use rusqlite::{..., params_from_iter};

...
let mut stmt = match lock.prepare(query) {
    Ok(stmt) => stmt,
    _ => return Err(()),
};

let mut rows = match stmt.query(params_from_iter(params)) {
    Ok(rows) => rows,
    _ => return Err(()),
};
```

我们已经得到查询结果，现在需要把结果转换为一个 `(u16, String)` 类型的元组 再把他们汇集到一个向量里返回

```rs
...
let mut pokemon_rows = vec![];

while let Ok(Some(row)) = rows.next() {
    match (row.get::<usize, u16>(0), row.get::<usize, String>(1)) {
        (Ok(number), Ok(name)) => pokemon_rows.push((number, name)),
        _ => return Err(()),
    };
}

Ok(pokemon_rows)
```

对 `types` 表也一样，这个辅助函数接收一个数据库连接和一个 `number`，查询成功会返回一个字符串向量，表示某一只宝可梦的类型：

```rs
fn fetch_type_rows(lock: &MutexGuard<'_, Connection>, number: u16) -> Result<Vec<String>, ()> {
    // code will go here
}
```
准备查询语句，带着参数进行查询：
```rs
let mut stmt = match lock.prepare("select name from types where pokemon_number = ?") {
    Ok(stmt) => stmt,
    _ => return Err(()),
};

let mut rows = match stmt.query([number]) {
    Ok(rows) => rows,
    _ => return Err(()),
};
```
依次从结果中提取出类型：
```rs
let mut type_rows = vec![];

while let Ok(Some(row)) = rows.next() {
    match row.get::<usize, String>(0) {
        Ok(name) => type_rows.push(name),
        _ => return Err(()),
    };
}
Ok(type_rows)
```
Aaaand，繁荣完成！这两个功能现在都实现了。使用它们去实现 fetch_one 和 fetch_all 会更容易:)

## 查询一只宝可梦
我们会一步一步来，首先处理下面的方法：

```rs
fn fetch_one(&self, number: PokemonNumber) -> Result<Pokemon, FetchOneError> {
    // code will go here
}
```
首先，我们要先拿到锁，并通过调用之前的辅助函数去查询宝可梦：
```rs
let lock = match self.connection.lock() {
    Ok(lock) => lock,
    _ => return Err(FetchOneError::Unknown),
};

let mut pokemon_rows = match Self::fetch_pokemon_rows(&lock, Some(u16::from(number.clone()))) {
    Ok(pokemon_rows) => pokemon_rows,
    _ => return Err(FetchOneError::Unknown),
};
```
当查询结果为空时，我们就返回 NotFound，否则返回查询结果的第一个：
```rs
...
if pokemon_rows.is_empty() {
    return Err(FetchOneError::NotFound);
}

let pokemon_row = pokemon_rows.remove(0);
```
不错。现在我们去查询宝可梦类型：
```rs
let type_rows = match Self::fetch_type_rows(&lock, pokemon_row.0) {
    Ok(type_rows) => type_rows,
    _ => return Err(FetchOneError::Unknown),
};
```
我们已经有口袋妖怪的编号、名称和类型。总是时候把他们封装为 Response 了：
```rs
...
match (
    PokemonNumber::try_from(pokemon_row.0),
    PokemonName::try_from(pokemon_row.1),
    PokemonTypes::try_from(type_rows),
) {
    (Ok(number), Ok(name), Ok(types)) => Ok(Pokemon::new(number, name, types)),
    _ => Err(FetchOneError::Unknown),
}
```
## 查询所有宝可梦
```rs
fn fetch_all(&self) -> Result<Vec<Pokemon>, FetchAllError> {
    // code will go here
}
```
首先获取锁并查询所有宝可梦：
```rs
let lock = match self.connection.lock() {
    Ok(lock) => lock,
    _ => return Err(FetchAllError::Unknown),
};

let pokemon_rows = match Self::fetch_pokemon_rows(&lock, None) {
    Ok(pokemon_rows) => pokemon_rows,
    _ => return Err(FetchAllError::Unknown),
};
```
你可以注意到，我们把 number 参数设置为了空，对每个宝可梦，我们会单独查询它的类型，最终封装为一个列表：
```rs
...
let mut pokemons = vec![];

for pokemon_row in pokemon_rows {
    let type_rows = match Self::fetch_type_rows(&lock, pokemon_row.0) {
        Ok(type_rows) => type_rows,
        _ => return Err(FetchAllError::Unknown),
    };

    let pokemon = match (
        PokemonNumber::try_from(pokemon_row.0),
        PokemonName::try_from(pokemon_row.1),
        PokemonTypes::try_from(type_rows),
    ) {
        (Ok(number), Ok(name), Ok(types)) => Pokemon::new(number, name, types),
        _ => return Err(FetchAllError::Unknown),
    };

    pokemons.push(pokemon);
}

Ok(pokemons)
```
## 插入一只宝可梦
```rs
fn insert(
    &self,
    number: PokemonNumber,
    name: PokemonName,
    types: PokemonTypes,
) -> Result<Pokemon, InsertError> {
    // code will go here
}
```
首先，我们从连接中获取锁：
```rs
let mut lock = match self.connection.lock() {
    Ok(lock) => lock,
    _ => return Err(InsertError::Unknown),
};
```
接着我们创建了一个事务。为什么我们没有直接执行一条命令？因为我们需要向 pokemons 中插入一只宝可梦，同时要想 types 中1插入它的类型。如果这两次插入有一个失败了，我们希望这次插入能够成功回滚。使用事务时，你需要先创建 SQL 语句，之后提交。如果有错误发生rusqlite 会自动完成回滚。

```rs
let transaction = match lock.transaction() {
    Ok(transaction) => transaction,
    _ => return Err(InsertError::Unknown),
};
```
现在我们要向事务中添加第一条命令。我们要插入一只宝可梦。如果已经存在，我们希望函数执行失败并返回一个错误：
```rs
use rusqlite::{..., Error::SqliteFailure, params};

...
match transaction.execute(
    "insert into pokemons (number, name) values (?, ?)",
    params![u16::from(number.clone()), String::from(name.clone())],
) {
    Ok(_) => {}
    Err(SqliteFailure(_, Some(message))) => {
        if message == "UNIQUE constraint failed: pokemons.number" {
            return Err(InsertError::Conflict);
        } else {
            return Err(InsertError::Unknown);
        }
    }
    _ => return Err(InsertError::Unknown),
};
```
在这里，我们使用 rusqlite 返回的错误信息来检查错误是否是由于冲突引起的。现在宝可梦的插入逻辑完成了。

接下来要处理插入类型。我们需要为 PokemonTypes 参数中的每种类型都分别执行依次插入操作：
```rs
...
for _type in Vec::<String>::from(types.clone()) {
    if let Err(_) = transaction.execute(
        "insert into types (pokemon_number, name) values (?, ?)",
        params![u16::from(number.clone()), _type],
    ) {
        return Err(InsertError::Unknown);
    }
}
```
现在，我们可以提交这个事物，并返回响应如果一切正常的话。
```rs
...
match transaction.commit() {
    Ok(_) => Ok(Pokemon::new(number, name, types)),
    _ => Err(InsertError::Unknown),
}
```

## 删除一只宝可梦

```rs
fn delete(&self, number: PokemonNumber) -> Result<(), DeleteError> {
    // code will go here
}
```
```rs
let lock = match self.connection.lock() {
    Ok(lock) => lock,
    _ => return Err(DeleteError::Unknown),
};
```
```rs
match lock.execute(
    "delete from pokemons where number = ?",
    params![u16::from(number)],
) {
    Ok(0) => Err(DeleteError::NotFound),
    Ok(_) => Ok(()),
    _ => Err(DeleteError::Unknown),
}
```
这里需要注意两点：第一，我们不需要再关注删除宝可梦类型信息，因为再创建数据表时我们已经设置了 `on delete cascade` 让 sqlite 去自动处理。第二，我们使用删除返回的删除行数去判断是否删除成功，如果数量是 0，就表示没有一行数据被成功删除。

你现在应该能使用你的 SQLite 数据库作为存储库了，而且能通过 CLI 和 HTTP API 两种方式访问。你可以尝试从 CLI 创建一些新的宝可梦，并从 HTTP API 去获取他们 :)

## 插曲
