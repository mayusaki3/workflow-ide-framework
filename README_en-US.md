[日本語](./README.md) | [English](./README_en-US.md)

# Workflow IDE Framework

The Workflow IDE Framework is a Rust-based IDE platform framework for building IDE-style applications.  
It is designed to facilitate the development of IDE applications with dock-style and scene-style UIs that run on Windows, macOS, and Linux.

## Purpose

It provides a common foundation for building the following types of IDE-style applications:

- Runtime IDE
- Simulation IDE
- AI Studio
- Asset Studio
- Workflow IDE
- GPU Viewport IDE

## Features

- Dock-style UI
- Scene switching
- Page-based UI
- GPU Viewport
- Runtime separation
- Asynchronous tasks
- Event bus
- UI API
- Workspace / Session Saving
- Command System

## UI Structure

```text
Page
  ↓
Tab
  ↓
Frame
  ↓
Scene
```

## Expected Architecture

```text
IDE Product
  ├─ workflow-ide-framework
  ├─ Runtime
  ├─ SDK
  ├─ Domain Model
  └─ Application Pages
```

## Documentation

- [Table of Contents](./docs/ja-JP/目次.md) (Japanese)

## Documentation Specifications

We will use HLDocS for documentation.

- https://github.com/mayusaki3/HLDocS

---
