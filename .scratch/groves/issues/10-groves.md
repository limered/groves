# 10: Groves

**Parent:** spec 015

**What to build:** Nine numbered **Groves**, switched with `Alt+1..9`. `Alt+Shift+1..9` moves the focused pane to another grove, and `Alt+Shift+h/j/k/l` swaps panes. A thin status bar shows the current grove, `Alt+.` shows an overlay listing every key, and an empty grove shows the hint "`Alt+Enter` open repo · `Alt+.` keys". A repository is open in at most one pane across all groves.

**Blocked by:** 09

**Status:** ready-for-agent

**Rust focus:** fixed-size arrays of structs, moving owned values between containers (`std::mem::take`/`swap`), keymap as data (a table of key → action enum).

- [ ] Moving a pane between groves keeps both trees valid (tests)
- [ ] Swapping panes (tests)
- [ ] The duplicate-repo rule holds across groves (tests)
- [ ] Manual: status bar, overlay, empty hint
