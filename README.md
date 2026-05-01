<div align="center">
    <h1>cahotic</h1>
    <b><p>Thread Pool Management</p></b>
    <p>⚙️ under development ⚙️</p>
    <b>
        <p>version / 0.4.0</p>
    </b>
</div>

## About
`cahotic`, thread pool management written in rust.

## Vesrion
what's new with: [version/0.4.0](https://github.com/araxnoid-code/cahotic/blob/version/0.4.0/version.md)

## Guide 
explanation of main features (English and Indonesian available): [guide.md](https://github.com/araxnoid-code/cahotic/blob/version/0.4.0/guide/guide.md)

## Starting
### Installation
Run the following Cargo command in your project directory:
```sh
cargo add cahotic
```
Or add the following line to your Cargo.toml:
```toml
cahotic = "0.4.0"
```

### Code
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<Option<i32>>().build().unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        DefaultOutput(None)
    }));

    cahotic.join();
}
```
