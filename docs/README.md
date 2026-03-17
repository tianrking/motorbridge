# Documentation Hub

- English: [en/index.md](en/index.md)
- 中文: [zh/index.md](zh/index.md)

The bilingual docs under `docs/en` and `docs/zh` are the only maintained documentation structure.

## Docs Map

```mermaid
flowchart LR
  HUB["docs/README.md"] --> EN["docs/en/index.md"]
  HUB --> ZH["docs/zh/index.md"]
  EN --> EN_ARCH["en/architecture.md"]
  EN --> EN_CLI["en/cli.md"]
  EN --> EN_ABI["en/abi.md"]
  EN --> EN_EX["en/examples.md"]
  EN --> EN_DEV["en/devices.md"]
  EN --> EN_EXT["en/extending.md"]
  ZH --> ZH_ARCH["zh/architecture.md"]
  ZH --> ZH_CLI["zh/cli.md"]
  ZH --> ZH_ABI["zh/abi.md"]
  ZH --> ZH_EX["zh/examples.md"]
  ZH --> ZH_DEV["zh/devices.md"]
  ZH --> ZH_EXT["zh/extending.md"]
```
