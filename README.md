<h1 align="center">Resilient-rs</h1>
<div align="center">

<i>A Rust utility library for fault tolerance, including retry strategies, backoff mechanisms, failure handling and much more.</i>
<br>
<br>
<a href="https://discord.com/invite/BymX4aJeEQ"><img src="https://img.shields.io/discord/733027681184251937.svg?style=flat&label=Join%20Community&color=7289DA" alt="Join Community Badge"/></a>
<a href="https://github.com/semicolon-10/resilient-rs/graphs/contributors"><img alt="GitHub contributors" src="https://img.shields.io/github/contributors/semicolon-10/resilient-rs.svg"></a>
[![Crates.io](https://img.shields.io/crates/v/resilient-rs.svg)](https://crates.io/crates/resilient-rs)
[![Downloads](https://img.shields.io/crates/d/resilient-rs)](https://crates.io/crates/resilient-rs)
[![Docs.rs](https://docs.rs/resilient-rs/badge.svg)](https://docs.rs/resilient-rs/latest/resilient_rs/)
<br>
<br>
<i>💖 Loved the work? [Subscribe to my YouTube channel](https://www.youtube.com/@Semicolon10) or consider giving this repository a ⭐ to show your support!</i>
</div>


## 🚀 Feature Overview

Here’s a snapshot of what this library brings to the table—resilience, reliability, and a sprinkle of magic! Check out the features, their details, and where they stand:

| **Feature**        | **Details**                                | **Status**          |
|--------------------|--------------------------------------------|---------------------|
| **🔄 Retry**       | Basic retry functionality—keeps trying!    | ✅ **Stable**       |
|                    | With Backoff (exponential)—smart delays    | ✅ **Stable**       |
|                    | With Fallback—graceful recovery            | ✅ **Stable**       |
| **⚡ Execute**     | Run ops with timeout & fallback—like a pro | ✅ **Stable**       |
| **🧵 Parallel Exec**       | Concurrent task execution with limits      | 🛠️ **Planned**       |
| 🛡️ Circuit Breaker | Stops cascading chaos in its tracks | ⚠️ Thread Unsafe |
| **📦 Memoize**     | Future Cache                               | 🛠️ **Planned**     |
| **📜 Logging**     | Debug like a detective—full support        | ✅ **Stable**       |
| **📚 More Examples** | Extra demos to spark your imagination      | 🛠️ **Planned**     |

### Notes:
- **Supported Contexts**: All features work seamlessly for both **synchronous** and **asynchronous** operations—flexibility is our middle name!

## 🏃‍♂️ Runtime Compatibility

This library plays nice with your favorite Rust async runtimes. The `resilient_rs::asynchronous` module has you covered with:

- **[Tokio](https://crates.io/crates/tokio)** - Power up with Tokio’s async I/O and runtime
- **[async-std](https://crates.io/crates/async-std)** - Keep it light with async-std’s sleek runtime
- **[futures](https://crates.io/crates/futures)** - Stick to the basics with the core futures crate and blocking execution

---
## 📦 How to Use `resilient-rs`

Here’s a quick example of how to use the `resilient-rs` crate in your Rust project.

### 1️⃣ Add `resilient-rs` to Your `Cargo.toml`

Add the following line to your `Cargo.toml` file:

```toml
[dependencies]
resilient-rs = "0.4.7" # Replace with the latest version
```
OR
```bash
cargo add resilient-rs
```

## 📖 Examples

Hover over the function you want to use in your IDE to see code documentation examples, or check out the [code-examples](https://github.com/semicolon-10/resilient-rs/tree/main/code-examples/) folder for example usage of this crate.

## 🚀 Contributing

We welcome your contributions! Please check out our [Contributing Guidelines](https://github.com/semicolon-10/resilient-rs/blob/main/CONTRIBUTING.md) to get started.