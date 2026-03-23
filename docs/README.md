# Documentation Hub

- English: [en/index.md](en/index.md)
- 中文: [zh/index.md](zh/index.md)

The bilingual docs under `docs/en` and `docs/zh` are the maintained documentation structure.

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
  EN --> EN_WIN["en/windows_distribution.md"]
  EN --> EN_TST["en/testing.md"]
  EN --> EN_CANDBG["en/can_debugging.md"]
  ZH --> ZH_ARCH["zh/architecture.md"]
  ZH --> ZH_CLI["zh/cli.md"]
  ZH --> ZH_ABI["zh/abi.md"]
  ZH --> ZH_EX["zh/examples.md"]
  ZH --> ZH_DEV["zh/devices.md"]
  ZH --> ZH_EXT["zh/extending.md"]
  ZH --> ZH_WIN["zh/windows_distribution.md"]
  ZH --> ZH_TST["zh/testing.md"]
  ZH --> ZH_CANDBG["zh/can_debugging.md"]
```

## Quick Entry

- EN unified scan: [`docs/en/cli.md`](en/cli.md)
- 中文统一扫描: [`docs/zh/cli.md`](zh/cli.md)
- EN Windows distribution: [`docs/en/windows_distribution.md`](en/windows_distribution.md)
- 中文 Windows 分发: [`docs/zh/windows_distribution.md`](zh/windows_distribution.md)
- EN CAN debugging (slcan + pcan): [`docs/en/can_debugging.md`](en/can_debugging.md)
- 中文 CAN 调试（slcan + pcan）: [`docs/zh/can_debugging.md`](zh/can_debugging.md)
- EN testing guide: [`docs/en/testing.md`](en/testing.md)
- 中文测试指南: [`docs/zh/testing.md`](zh/testing.md)
