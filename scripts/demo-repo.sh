#!/bin/sh
# Builds the demo git repository used for live runs of fmc and of the parity reference (Scouter).
# The result is deterministic: same files, same commit hashes, same `git status` on every run.
# Usage: scripts/demo-repo.sh <dir>
set -eu

DIR=${1:?usage: scripts/demo-repo.sh <dir>}
MARKER=.git/fmc-demo-repo

# Only replace a directory this script made, or an empty one: never delete someone's work.
if [ -e "$DIR" ]; then
    if [ -f "$DIR/$MARKER" ]; then
        rm -rf "$DIR"
    elif [ -d "$DIR" ] && [ -z "$(ls -A "$DIR")" ]; then
        rmdir "$DIR"
    else
        echo "error: $DIR exists and was not made by this script; refusing to replace it" >&2
        exit 1
    fi
fi
mkdir -p "$DIR"
cd "$DIR"

export GIT_AUTHOR_NAME="Hermione Granger" GIT_AUTHOR_EMAIL="hermione.granger@example.com"
export GIT_COMMITTER_NAME="Hermione Granger" GIT_COMMITTER_EMAIL="hermione.granger@example.com"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
g() { git -c init.defaultBranch=main -c commit.gpgsign=false -c core.autocrlf=false "$@"; }
commit() {
    GIT_AUTHOR_DATE="$1" GIT_COMMITTER_DATE="$1" g commit -q -m "$2"
}

g init -q
: > "$MARKER"
mkdir -p src/ledger src/owl scripts docs/guide tests assets notes target/debug node_modules/left-pad

cat > '.gitignore' <<'FMC_EOF'
target/
node_modules/
*.log
.env
FMC_EOF

cat > 'README.md' <<'FMC_EOF'
# Marauder's Map

A small ledger and symbology toolkit for the Ministry of Magic archive.

## Quick start

1. Install the toolkit with `cargo build`.
2. Open the vault ledger with `vault open --key 713`.
3. Read the symbology notes in `scripts/symbology.py`.

## Modules

| Module             | Language | Owner             |
| ------------------ | -------- | ----------------- |
| `src/main.rs`      | Rust     | Hermione Granger  |
| `src/ledger/`      | TS       | Bill Weasley      |
| `scripts/`         | Python   | Robert Langdon    |

## Usage

```rust
fn main() {
    println!("I solemnly swear that I am up to no good.");
}
```

Run the ledger from TypeScript:

```ts
import { openVault } from "./src/ledger/vault";

const vault = openVault("gringotts-713");
console.log(vault.balance);
```

> The map shows every room of the castle, and it never lies.
> It is read-only; it only reports what is already there.

## Roadmap

- [x] Vault ledger in TypeScript
- [x] Symbology helper in Python
- [ ] Chamber 13 audit trail
- [ ] Sync with the Room of Requirement

## Data flow

```mermaid
flowchart TD
    A[Owl post arrives] --> B{Vault key valid?}
    B -- yes --> C[Open ledger]
    B -- no --> D[Reject and log]
    C --> E[(Gringotts store)]
```

```mermaid
sequenceDiagram
    participant Harry
    participant Ledger
    participant Vault
    Harry->>Ledger: openVault("gringotts-713")
    Ledger->>Vault: read balance
    Vault-->>Ledger: 1000 galleons
    Ledger-->>Harry: balance ok
```

See the [Ministry archive](https://example.com/ministry) and the
[Da Vinci notes][davinci] for background.

[davinci]: https://example.com/davinci-notes

---

Learn more in [docs/guide/lumos.md](docs/guide/lumos.md).
FMC_EOF

cat > 'src/main.rs' <<'FMC_EOF'
use std::collections::HashMap;

struct Vault {
    key: u32,
    balance: i64,
}

fn open(key: u32) -> Vault {
    Vault { key, balance: 1000 }
}

fn main() {
    let mut owners: HashMap<&str, &str> = HashMap::new();
    owners.insert("Harry", "Gryffindor");
    let vault = open(713);
    println!("Vault {} holds {} galleons", vault.key, vault.balance);
}
FMC_EOF

cat > 'src/ledger/vault.ts' <<'FMC_EOF'
export interface Vault {
  id: string;
  balance: number;
}

export function openVault(id: string): Vault {
  return { id, balance: 1000 };
}

export function deposit(vault: Vault, amount: number): Vault {
  return { ...vault, balance: vault.balance + amount };
}
FMC_EOF

cat > 'src/owl/post.go' <<'FMC_EOF'
package owl

import "fmt"

// Post carries a letter from Hogwarts to Diagon Alley.
type Post struct {
	From string
	To   string
}

func (p Post) String() string {
	return fmt.Sprintf("%s -> %s", p.From, p.To)
}
FMC_EOF

cat > 'scripts/symbology.py' <<'FMC_EOF'
from dataclasses import dataclass


@dataclass
class Symbol:
    name: str
    meaning: str


def decode(symbols: list[Symbol]) -> dict[str, str]:
    return {symbol.name: symbol.meaning for symbol in symbols}


if __name__ == "__main__":
    print(decode([Symbol("rose", "the Rose Line")]))
FMC_EOF

cat > 'docs/guide/lumos.md' <<'FMC_EOF'
# Lumos

Point your wand and say the words.

- Light the path
- Keep the wand steady
FMC_EOF

cat > 'docs/guide/alohomora.md' <<'FMC_EOF'
# Alohomora

Unlocks doors and windows. It does not work on the vaults at Gringotts.
FMC_EOF

cat > 'tests/notes.txt' <<'FMC_EOF'
placeholder
FMC_EOF

g add -A
commit "2026-11-26T03:19:16+07:00" "Initial archive"

base64 -d > assets/marauder-map.png <<'FMC_EOF'
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
jHNGknHJSDOuGVkGGXnGLaPIuGeUGY+MKuOZUWe8MpqMd0ab8cnoMr4ZfcYv+w/kDHLEnGGOlDPK
kXPGOUrOJEfNmeZoObMcPWeeY+QscsycZY6Vs8qxc9Y5Ts4mx83Z5ng5uxw/Z58T5BxywpxjTpRz
yolzzjlJziUnzbnmZDnk5Dm3nCLnnlPmPHKqnGdOnfPKaXLeOW3OJ6fL+eb0Ob/8P1AwKBALhgVS
wahALhgXKAWTArVgWqAVzAr0gnmBUbAoMAuWBVbBqsAuWBc4BZsCt2Bb4BXsCvyCfUFQcCgIC44F
UcGpIC44FyQFl4K04FqQFVCQF9wKioJ7QVnwKKgKngV1waugKXgXtAWfgq7gW9AX/Ir/QMmgRCwZ
lkgloxK5ZFyilExK1JJpiVYyK9FL5iVGyaLELFmWWCWrErtkXeKUbErckm2JV7Ir8Uv2JUHJoSQs
OZZEJaeSuORckpRcStKSa0lWQklecispSu4lZcmjpCp5ltQlr5Km5F3SlnxKupJvSV/yK/8DFYMK
sWJYIVWMKuSKcYVSMalQK6YVWsWsQq+YVxgViwqzYllhVawq7Ip1hVOxqXArthVexa7Cr9hXBBWH
irDiWBFVnCriinNFUnGpSCuuFVkFFXnFraKouFeUFY+KquJZUVe8KpqKd0Vb8anoKr4VfcWv+g/U
DGrEmmGNVDOqkWvGNUrNpEatmdZoNbMavWZeY9QsasyaZY1Vs6qxa9Y1Ts2mxq3Z1ng1uxq/Zl8T
1BxqwppjTVRzqolrzjVJzaUmrbnWZDXU5DW3mqLmXlPWPGqqmmdNXfOqaWreNW3Np6ar+db0Nb/6
P9AwaBAbhg1Sw6hBbhg3KA2TBrVh2qA1zBr0hnmD0bBoMBuWDVbDqsFuWDc4DZsGt2Hb4DXsGvyG
fUPQcGgIG44NUcOpIW44NyQNl4a04dqQNdCQN9waioZ7Q9nwaKgang11w6uhaXg3tA2fhq7h29A3
/Jr/QMugRWwZtkgtoxa5ZdyitExa1JZpi9Yya9Fb5i1Gy6LFbFm2WC2rFrtl3eK0bFrclm2L17Jr
8Vv2LUHLoSVsObZELaeWuOXckrRcWtKWa0vWQkvecmspWu4tZcujpWp5ttQtr5am5d3StnxaupZv
S9/ya/8DHYMOsWPYIXWMOuSOcYfSMelQO6YdWsesQ++Ydxgdiw6zY9lhdaw67I51h9Ox6XA7th1e
x67D79h3BB2HjrDj2BF1nDrijnNH0nHpSDuuHVkHHXnHraPouHeUHY+OquPZUXe8OpqOd0fb8eno
Or4dfcev+w/0DHrEnmGP1DPqkXvGPUrPpEftmfZoPbMevWfeY/QsesyeZY/Vs+qxe9Y9Ts+mx+3Z
9ng9ux6/Z98T9Bx6wp5jT9Rz6ol7zj1Jz6Un7bn2ZD305D23nqLn3lP2PHqqnmdP3fPqaXrePW3P
p6fr+fb0Pb+eP6dUcMTeNaiUAAAAAElFTkSuQmCC
FMC_EOF
g add assets/marauder-map.png
commit "2026-11-26T03:19:27+07:00" "Add map image"

# Working tree: every change kind the changes screen shows.
cat > 'src/ledger/vault.ts' <<'FMC_EOF'
export interface Vault {
  readonly kind: "vault";
  id: string;
  balance: number;
}

export function openVault(id: string): Vault {
  return { kind: "vault", id, balance: 1000 };
}

export function deposit(vault: Vault, amount: number): Vault {
  return { ...vault, balance: vault.balance + amount };
}
FMC_EOF
g add src/ledger/vault.ts
cat > 'src/ledger/vault.ts' <<'FMC_EOF'
export interface Vault {
  readonly kind: "vault";
  id: string;
  balance: number;
  currency: "galleon";
}

export function openVault(id: string): Vault {
  return { kind: "vault", id, balance: 1000, currency: "galleon" };
}

export function deposit(vault: Vault, amount: number): Vault {
  return { ...vault, balance: vault.balance + amount };
}
FMC_EOF
cat > 'scripts/symbology.py' <<'FMC_EOF'
from dataclasses import dataclass


@dataclass
class Symbol:
    name: str
    meaning: str


def decode(symbols: list[Symbol]) -> dict[str, str]:
    return {symbol.name: symbol.meaning for symbol in symbols}


if __name__ == "__main__":
    print(decode([Symbol("rose", "the Rose Line"), Symbol("owl", "post")]))
FMC_EOF
g add scripts/symbology.py
g rm -q docs/guide/lumos.md
g mv docs/guide/alohomora.md docs/guide/unlocking.md
rm tests/notes.txt
cat > 'README.md' <<'FMC_EOF'
# Marauder's Map

A small ledger and symbology toolkit for the Ministry of Magic archive.

## Quick start

1. Install the toolkit with `cargo build`.
2. Open the vault ledger with `vault open --key 713`.
3. Read the symbology notes in `scripts/symbology.py`.

## Modules

| Module             | Language | Owner             |
| ------------------ | -------- | ----------------- |
| `src/main.rs`      | Rust     | Hermione Granger  |
| `src/ledger/`      | TS       | Bill Weasley      |
| `scripts/`         | Python   | Robert Langdon    |

## Usage

```rust
fn main() {
    println!("I solemnly swear that I am up to no good.");
}
```

Run the ledger from TypeScript:

```ts
import { openVault } from "./src/ledger/vault";

const vault = openVault("gringotts-713");
console.log(vault.balance);
```

> The map shows every room of the castle, and it never lies.
> It is read-only; it only reports what is already there.

## Roadmap

- [x] Vault ledger in TypeScript
- [x] Symbology helper in Python
- [ ] Chamber 13 audit trail
- [ ] Sync with the Room of Requirement

## Data flow

```mermaid
flowchart TD
    A[Owl post arrives] --> B{Vault key valid?}
    B -- yes --> C[Open ledger]
    B -- no --> D[Reject and log]
    C --> E[(Gringotts store)]
```

```mermaid
sequenceDiagram
    participant Harry
    participant Ledger
    participant Vault
    Harry->>Ledger: openVault("gringotts-713")
    Ledger->>Vault: read balance
    Vault-->>Ledger: 1000 galleons
    Ledger-->>Harry: balance ok
```

See the [Ministry archive](https://example.com/ministry) and the
[Da Vinci notes][davinci] for background.

[davinci]: https://example.com/davinci-notes

---

Learn more in [docs/guide/lumos.md](docs/guide/lumos.md).

## Changelog

- Added the Marauder map image.
FMC_EOF
cat > 'src/main.rs' <<'FMC_EOF'
use std::collections::HashMap;

struct Vault {
    key: u32,
    balance: i64,
    holder: String,
}

fn open(key: u32) -> Vault {
    Vault { key, balance: 1000, holder: String::from("Harry Potter") }
}

fn main() {
    let mut owners: HashMap<&str, &str> = HashMap::new();
    owners.insert("Harry", "Gryffindor");
    let vault = open(713);
    println!("Vault {} held by {} holds {} galleons", vault.key, vault.holder, vault.balance);
}
FMC_EOF
cat > 'src/ledger/chamber13.ts' <<'FMC_EOF'
export const chamber = 13;

export function audit(): string {
  return "Chamber 13 audit trail";
}
FMC_EOF
cat > 'notes/owl-post.md' <<'FMC_EOF'
# Notes

Talk to Dumbledore about the Room of Requirement.
FMC_EOF

# 4 KiB binary with NUL bytes: 0..255 repeated 16 times.
byte=0
pattern=
while [ "$byte" -lt 256 ]; do
    pattern="$pattern\\$(printf %03o "$byte")"
    byte=$((byte + 1))
done
i=0
while [ "$i" -lt 16 ]; do
    printf "$pattern"
    i=$((i + 1))
done > assets/seal.bin

# Ignored files and edge cases: a large log, a wide line, non-UTF-8 text, symlinks (one a loop).
printf 'log line\n' > app.log
printf 'module\n' > node_modules/left-pad/index.js
printf 'built\n' > target/debug/marauder
awk 'BEGIN { for (i = 0; i < 9000; i++) { for (j = 0; j < 12; j++) printf "Expelliarmus! "; printf "\n" } }' > target/debug/big.log
awk 'BEGIN { printf "Hermione Granger:"; for (i = 0; i < 61; i++) printf " Accio-%03d", i; printf "\n" }' > target/debug/wide.txt
printf 'Caf\351 au lait \377\376 in Hogsmeade\n' > target/debug/latin1.txt
ln -s .. target/debug/loop
ln -s ../../README.md target/debug/readme-link

echo "Demo repository ready at $DIR"
