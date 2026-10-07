# Third-party notices

The bundled Windows renderer includes Lua and Rust packages. This skill's own code and documentation are covered by its [MIT license](LICENSE); each dependency retains its own license. Exact Rust package versions are pinned in [Cargo.lock](scripts/engine/Cargo.lock).

| Component | License | Source |
| --- | --- | --- |
| Lua 5.4 | MIT | [Lua](https://www.lua.org/license.html) |
| lua-src | MIT | [lua-src-rs](https://github.com/mlua-rs/lua-src-rs) |
| mlua and mlua-sys | MIT | [mlua](https://github.com/mlua-rs/mlua) |
| Other Rust packages | MIT, Apache-2.0, Unlicense, or Unicode-3.0 as declared by each package | Package names and versions in Cargo.lock; license metadata at [crates.io](https://crates.io/) |
| Pillow (installed separately) | HPND | [Pillow](https://github.com/python-pillow/Pillow/blob/main/LICENSE) |

The Lua and Rust packages are distributed under their respective license terms. The following MIT notice covers the Lua runtime and its Rust bindings, including the copyright notices shipped with `lua-src` and `mlua`:

> Copyright © 1994–2017 Lua.org, PUC-Rio.
> Copyright (c) 2020 Aleksandr Orlenko.
> Copyright (c) 2019–2021 A. Orlenko.
> Copyright (c) 2017 rlua.
>
> Permission is hereby granted, free of charge, to any person obtaining a copy
> of this software and associated documentation files (the "Software"), to deal
> in the Software without restriction, including without limitation the rights
> to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
> copies of the Software, and to permit persons to whom the Software is
> furnished to do so, subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all
> copies or substantial portions of the Software.
>
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
> IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
> FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
> AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
> LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
> OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
> SOFTWARE.
