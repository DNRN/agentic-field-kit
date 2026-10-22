#!/usr/bin/env python3
"""Locate the `git commit` a Claude Code PreToolUse Bash payload would run.

Reads the hook JSON on stdin. When the command runs a git commit that is not
bypassed with -n/--no-verify, prints two lines and exits 0:

    <directory git runs in>
    <diff kind>     --cached, or HEAD for `commit -a/--all`

Exits 1 otherwise: no commit, bypassed, or unparseable input (fail-open).

Handles: `&&`/`||`/`;`/`|`/newline chains, `cd dir`, `git -C dir`,
git global options (`-c k=v`, `--no-pager`, ...), env assignments and
command prefixes, `sh -c '...'`, combined short flags (`-am`), option values
(a message containing `-n` is a message, not a flag), and heredoc bodies.
"""
import json
import os
import re
import shlex
import sys

SEPARATOR_CHARS = set("();|&\n")
SHELLS = {"sh", "bash", "zsh", "dash", "ksh"}
PREFIX_WORDS = {"if", "then", "else", "elif", "do", "while", "until", "!", "{",
                "time", "command", "exec", "nohup", "env", "sudo", "xargs"}
ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
# git global options whose value is the next argument.
GIT_GLOBAL_WITH_VALUE = {"-C", "-c", "--git-dir", "--work-tree", "--namespace",
                         "--config-env"}
# `git commit` options whose value may be the next argument.
COMMIT_SHORT_WITH_VALUE = set("mFcCt")
COMMIT_SHORT_OPTIONAL_VALUE = set("uS")  # value only when attached
COMMIT_LONG_WITH_VALUE = {"--message", "--file", "--reuse-message",
                          "--reedit-message", "--author", "--date", "--template",
                          "--fixup", "--squash", "--cleanup", "--trailer",
                          "--pathspec-from-file"}
HEREDOC = re.compile(r"<<-?[ \t]*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")


def strip_heredoc_bodies(command):
    """Drop heredoc bodies: their text is data (often a commit message)."""
    pos = 0
    while True:
        match = HEREDOC.search(command, pos)
        if not match:
            return command
        body_start = command.find("\n", match.end())
        if body_start == -1:
            return command
        end = re.compile(r"^[ \t]*" + re.escape(match.group(2)) + r"[ \t]*$",
                         re.MULTILINE).search(command, body_start + 1)
        if not end:
            return command
        command = command[:body_start + 1] + command[end.end():]
        pos = body_start + 1


def tokenize(command):
    lexer = shlex.shlex(strip_heredoc_bodies(command), posix=True,
                        punctuation_chars="();<>|&\n")
    lexer.whitespace = " \t\r"
    lexer.whitespace_split = True
    return list(lexer)


def segments(tokens):
    """Split tokens into simple commands, dropping redirections."""
    current, skip_next = [], False
    for tok in tokens:
        if skip_next:
            skip_next = False
            continue
        if tok and set(tok) <= SEPARATOR_CHARS:
            if current:
                yield current
            current = []
        elif tok and set(tok) <= set("<>&"):
            skip_next = True  # redirection target
        else:
            current.append(tok)
    if current:
        yield current


def strip_prefix(seg):
    """Skip shell keywords, env assignments, and wrapper commands."""
    i, after_wrapper = 0, False
    while i < len(seg):
        tok = seg[i]
        if tok in PREFIX_WORDS:
            after_wrapper = True
        elif ASSIGNMENT.match(tok) or (after_wrapper and tok.startswith("-")):
            pass
        else:
            break
        i += 1
    return seg[i:]


def parse_commit_args(args):
    """Return (bypassed, all) for `git commit` arguments."""
    bypass, all_ = False, False
    i = 0
    while i < len(args):
        arg = args[i]
        i += 1
        if arg == "--":
            break
        if arg.startswith("--"):
            name = arg.split("=", 1)[0]
            if "=" in arg:
                continue
            if name in COMMIT_LONG_WITH_VALUE:
                i += 1
            elif len(name) >= len("--no-veri") and "--no-verify".startswith(name):
                bypass = True  # git accepts unambiguous abbreviations
            elif name == "--verify":
                bypass = False
            elif name == "--all":
                all_ = True
        elif arg.startswith("-") and len(arg) > 1:
            for j, flag in enumerate(arg[1:], start=1):
                if flag in COMMIT_SHORT_WITH_VALUE:
                    if j == len(arg) - 1:
                        i += 1  # value is the next argument
                    break  # rest of the cluster is the value
                if flag in COMMIT_SHORT_OPTIONAL_VALUE:
                    break
                if flag == "n":
                    bypass = True
                elif flag == "a":
                    all_ = True
    return bypass, all_


def find_commit(command, cwd, depth=0):
    """Return (dir, all) for the first non-bypassed commit, else None."""
    if depth > 3:
        return None
    for seg in segments(tokenize(command)):
        seg = strip_prefix(seg)
        if not seg:
            continue
        name = os.path.basename(seg[0])
        if name in ("cd", "pushd"):
            if len(seg) > 1 and seg[1] != "-":
                cwd = os.path.join(cwd, os.path.expanduser(seg[1]))
            elif len(seg) == 1:
                cwd = os.path.expanduser("~")
            continue
        if name in SHELLS:
            for k in range(1, len(seg) - 1):
                if re.fullmatch(r"-[a-zA-Z]*c[a-zA-Z]*", seg[k]):
                    found = find_commit(seg[k + 1], cwd, depth + 1)
                    if found:
                        return found
                    break
            continue
        if name != "git":
            continue
        git_dir, i = cwd, 1
        while i < len(seg) and seg[i].startswith("-"):
            if seg[i] == "-C" and i + 1 < len(seg):
                git_dir = os.path.join(git_dir, os.path.expanduser(seg[i + 1]))
            i += 2 if seg[i] in GIT_GLOBAL_WITH_VALUE else 1
        if i < len(seg) and seg[i] == "commit":
            bypass, all_ = parse_commit_args(seg[i + 1:])
            if not bypass:
                return os.path.normpath(git_dir), all_
    return None


def main():
    try:
        payload = json.load(sys.stdin)
        command = payload["tool_input"]["command"]
        cwd = payload.get("cwd") or os.getcwd()
        if not isinstance(command, str) or not isinstance(cwd, str):
            return 1
        found = find_commit(command, cwd)
    except (ValueError, KeyError, TypeError, AttributeError):
        return 1
    if not found:
        return 1
    directory, all_ = found
    sys.stdout.write(directory + "\n" + ("HEAD" if all_ else "--cached") + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
