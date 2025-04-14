# Hello
🧜‍♂️ a tauri hello project by sleet to interact with a hello smart contract on near

![img](DOCS/Screenshot.png)

---

### Dev and Build

```sh
# Install
pnpm install
pnpm tauri dev
# Error Check
npx tsc --noEmit --jsx react
cd src-tauri && cargo check
# Build
pnpm tauri build

# Cargo comands
cargo update
cargo clean
cargo check
```

mobile
```sh
pnpm tauri android init
pnpm tauri android dev
pnpm tauri icon src-tauri/icons/icon.png
pnpm tauri android build --aab
```


---


copyright 2025 by sleet.near
