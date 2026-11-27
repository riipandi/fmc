# Product

<!-- impeccable:product-schema 1 -->

## Platform

terminal. A native command-line app that draws a full-screen interface in the terminal. The first platform is macOS on Apple silicon.

## Stack

Rust, on OpenTUI (the native engine with its Rust binding). git on the host for the changes view.

## Users

Developers who open a folder in the terminal to read it: to look at a file, to read a README as it renders, to see what changed before they commit. Keyboard first; the mouse works too.

## Product Purpose

fmc lets you look at code and documents without opening an editor. Now: a read-only viewer with the same screens and keys as Scouter. Later: a file manager that can also create, move, and delete files, and open SQLite and DuckDB files.

Success: a developer runs `fmc` for every "what is in this folder" and "what did I change" moment, and it opens before they finish the thought.

## Positioning

A single fast binary with no runtime to install. It reads first and never changes your files until the file-manager milestone, and then only when you ask.

## Operating Context

Started from a shell inside a project (`fmc`, `fmc <folder>`, `fmc <file>`). Used next to an editor and to AI agents that change files underneath it, so it refreshes on its own.

## Capabilities and Constraints

- File tree with git status marks, icons, and ignored files dimmed.
- Reader for text with syntax colours in Scouter's languages and the owner's extra ones, Markdown rendered or as source, images, and a notice for binary files.
- Find a file by name, or search text inside files with a preview.
- Changes view: every changed file as a stacked diff, unified or split, with folding and hunk navigation.
- Light and dark palettes that follow the terminal background.
- Read-only. No editing.
- Not built yet: Mermaid diagrams, fuzzy finding, file operations, a database viewer, platforms other than macOS on Apple silicon.

## Brand Commitments

- Name: `fmc`, lowercase.
- Voice: plain, short, technical, no hype.

## Evidence on Hand

- No screenshots, measurements, users, or download counts exist yet. Do not invent any.

## Product Principles

1. Parity first. Until it matches Scouter, a difference from Scouter is a bug unless it was chosen on purpose.
2. Read first. The default is reading; changing files comes later and only on request.
3. Speed decides. Opening, moving, and scrolling never wait on disk or git.
4. Show, do not claim. Real captures and measured numbers only.

## Accessibility & Inclusion

Every action has a key. Both palettes keep readable text contrast, checked by a test. Colour is never the only signal: git states also show a letter.

## Roadmap

Parity with Scouter, then file operations, a SQLite and DuckDB viewer, and more platforms.
