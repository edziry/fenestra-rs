# WU-0014 registered evidence

This directory accepts the exact bounded artifact from the registered Windows
DX12 run defined by
[`docs/verification/WU-0014-windows-operator.md`](../../../docs/verification/WU-0014-windows-operator.md).

`windows-dx12-v1.txt` must be added only after the standalone verifier reports
`pass`. Synthetic fixtures and Linux developer controls do not belong here.

The recorded artifact was produced on Windows from source commit
`db0e86769950be8bb7387055f6eb3986062fc469` with a physical NVIDIA GeForce RTX
4060 selected through DX12. The independent Windows and Linux verifiers both
reported:

```text
pass|records=16|bytes=1216|generation=4
```

Its SHA-256 is
`7b856a334ba17de5415758daebd69af3b3ac84966021cdf884a342506d736af5`.
