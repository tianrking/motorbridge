# 分发渠道（CI 自动化）

本文说明在 GitHub Releases / PyPI 之外新增的分发渠道。

## 1) APT 仓库（GitHub Pages）

对应 workflow：`.github/workflows/apt-repo-publish.yml`

- 触发方式：
  - Release `published`
  - 或手动触发（`workflow_dispatch`）并传入 `tag`
- 输入资产：
  - 对应 Release 里的 `*.deb`
- 输出：
  - 在 `gh-pages` 分支发布 `/apt` 仓库目录
  - URL 形如：
    - `https://<owner>.github.io/<repo>/apt`

可选签名（生产建议开启）：

- `APT_GPG_PRIVATE_KEY`（ASCII armored 私钥）
- `APT_GPG_PASSPHRASE`（若私钥有口令）

如果未配置签名密钥，workflow 会走 `-skip-signing`（无签名元数据）。

## 2) Homebrew（本仓库内 Formula）

相关文件：

- `Formula/motor-cli.rb`
- `.github/workflows/release-macos-cli.yml`
- `.github/workflows/homebrew-formula-update.yml`

流程：

1. `release-macos-cli.yml` 构建并上传 `motor-cli-<tag>-macos-arm64.tar.gz` 到 Release。
2. `homebrew-formula-update.yml` 下载该包，计算 SHA256，并更新 `Formula/motor-cli.rb`。

使用示例：

```bash
brew tap tianrking/motorbridge
brew install motor-cli
```

## 3) Windows 包管理元数据

对应 workflow：`.github/workflows/windows-package-metadata.yml`

生成脚本：

- `tools/release/gen_windows_manifests.py`

生成结果：

- Scoop: `packaging/windows/scoop/motor-cli.json`
- Winget: `packaging/windows/winget/manifests/...`
- Chocolatey 模板：`packaging/windows/choco/*`

说明：

- Winget/Chocolatey 最终发布仍需走各自上游社区提交流程。
- 当前 workflow 负责在每次 Release 后自动同步仓库内元数据。
