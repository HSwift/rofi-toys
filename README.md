# rofi-toys

rofi 工具集合

## 构建

```sh
cargo build --release
```

## 产物

### clipd

剪贴板管理器, 记录剪贴板的历史记录以及恢复剪贴板内容, 支持文本/图片/HTML/URL格式, 额外提供 HTTP API 供 clipc 或其他工具, 默认监听 `$XDG_RUNTIME_DIR/clipd.sock`.

e.g.
```sh
curl --unix-socket /run/user/1000/clipd.sock http://localhost/list
```

详细 API 内容可自行翻阅源码.

### clipc

剪贴板管理前端, 可以用于设置当前剪贴板内容和浏览剪贴板历史.

### encoders

编码器工具集合, 包含 base64/url/html/unicode... 等等常用工具, 大部分工具基于剪贴板进行交互.

### containers

容器管理工具, 可以列出当前容器并显示详细信息, 例如 IP/hostname 等等. 根据 docker 配置, 可能需要给予二进制文件 sticky bit, 否则无法连接 docker api.

### notes

记事本工具，可以浏览或将剪贴板内容保存到记事本。
