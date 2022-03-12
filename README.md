# td: A simple todo CLI

Td is a tiny, easy to use task organization command line program written in Rust.

## Installation

```
cargo install --git https://github.com/Jummit/td.git
```

## Usage

Add a task to the list:

`td A new task`

The new task will be the first in the list.

Add multiple tasks:

`td One task, Another task, Third task`

Show all unfinished tasks:

`td`

Show tasks containing `coding`

`td show coding`

Start working on a task (push it to the top of the task list):

`td do <selector>`

Complete the current task:

`td done`

Complete all tasks containing `work:`:

`td done work:`

## Selectors

`td show` and `td do` both take "task selectors". The default behavior for `td show` is to show all tasks, and for `td do` it is to complete the last added task.

### Index Selectors

When displaying the tasks, they have a number next to them. Tasks can be selected by using these numbers or number ranges (`1-5`).

**Examples:**

```bash
td show 5
td do 2
td done 3-5
```

### Regex Selectors

If the selector is not a number or number range, it is compiled as a regex and matched against each task.

**Examples:**

```bash
td show "20**-01-03"
td do "*"
td done "(garden|house)"
```

## Task Files

The tasks are stored inside the OS-specific application data folder under `td-todo/tasks.csv`. This file contains the tasks and their created/completed times.

## Organization

Td doesn't have a native tag or grouping system, but one can easily emulated one by including tags inside the task description:

`td garden: Water plants`
`td garden: Pick cherries`

The tasks can then be listed using `td show`:

`td show garden`

```
1 garden: Pick cherries
2 garden: Water plants
```
