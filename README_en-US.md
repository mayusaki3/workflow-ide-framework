[日本語](./README.md) | [English](./README_en-US.md)

# Workflow IDE Framework

Workflow IDE Framework is an embeddable Rust framework for building IDE-style applications. A consumer application uses common Framework facilities such as panels, layout, projects, resources, and commands, while keeping domain-specific models and behavior in the application.

> **Status: v0.1.0 development**
>
> v0.1.0 is being prepared for publication. Windows, Linux, and macOS are target platforms. Items that have not been verified are explicitly published as `?` in the verification matrix.

## Intended uses

- Runtime / Simulation IDEs
- AI / Asset Studios
- Workflow Editors
- Development and validation applications with GPU viewports

## v0.1.0 scope

The implementation and v0.1.0 specification cover or are currently integrating:

- Application / Panel composition
- Dock / Layout foundation
- Text Editor / Log Viewer / Tree Viewer
- Flow / Graph Editor
- Property Panel
- Controller Panel
- Logging
- Language / Theme / Font settings
- Project persistence
- Resource Registry / File Resource Selector
- Application / Project / External resource scopes
- Resource filesystem operation consistency
- Notifications
- Extension boundaries for consumer applications

Some Project, Resource, Panel Container, and Notification work is still under implementation and verification. A feature listed here must not be interpreted as fully verified; see the verification documentation for current status.

## Responsibility boundary

The Framework owns generic IDE facilities and generic model/action/API boundaries. Consumer applications own domain models, domain validation, application-specific project data, resource semantics, and runtime-specific behavior.

Consumers are not expected to fork or directly modify Framework internals to integrate application-specific functionality.

## Target platforms

| OS | v0.1.0 policy |
| --- | --- |
| Windows | Verified on physical Windows systems |
| Linux | Verified where possible, including VM testing; physical-only items may remain unverified |
| macOS | Items that cannot currently be tested remain explicitly unverified |

Verification status is published as `○ / ✕ / ?`. Fine UI tuning is not a v0.1.0 release requirement; functional behavior, public APIs, persistence, state transitions, and consumer integration take priority.

## Development and public artifacts

`develop` is the v0.1.0 development branch. Development-history Example names such as `stepX_...` and technical validation records are kept as development material and will be separated from purpose-named consumer-facing Examples and documentation before release.

## Documentation

- [Documentation index](./docs/ja-JP/目次.md) (Japanese)
- [Requirements](./docs/ja-JP/01_要件定義/要件定義目次.md) (Japanese)
- [Specifications](./docs/ja-JP/02_仕様/仕様目次.md) (Japanese)
- [Technical verification](./docs/ja-JP/90_技術検証/技術検証目次.md) (Japanese)
- [v0.1.0 publication checklist](./docs/ja-JP/90_技術検証/v0.1.0_公開準備チェックリスト.md) (Japanese)

## UI details intentionally not frozen in v0.1.0

Fine defaults such as panel minimum sizes, split ratios, and notification presentation details are not treated as compatibility contracts in v0.1.0. They can be adjusted after real-world operation without changing the consumer API or project format.

## License

MIT License. See [LICENSE](./LICENSE).

## Documentation specification

Design and specification documents use [HLDocS](https://github.com/mayusaki3/HLDocS).
