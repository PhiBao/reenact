---
mode: testing
url: http://localhost:15323/auth
headless: true
max_steps: 40
tags: [reenact, issue-5462]
---

# Search filter persists after clicking Shortcuts even when search box is empty

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15323/.
Stay on localhost:15323 for the whole run and do not open any other website.
Do these in order:
1. Click the memo editor and type "apple pie"
2. Click the Save button to create the memo
3. Click the memo editor and type "banana bread"
4. Click the Save button to create the memo
5. Click the add (+) button next to Shortcuts in the sidebar
6. Type "Bananas" in the shortcut title field
7. Type `content.contains("banana")` in the shortcut filter field
8. Click the Save button in the shortcut dialog
9. Click the search box, type "apple" and press Enter
10. Click the "Bananas" shortcut in the sidebar
The reporter says the correct behavior is that After clicking the "Bananas" shortcut, the search box is empty and the memo list shows the "banana bread" memo (only the shortcut filter applies, so the earlier "apple" search no longer hides it). Verify that After clicking the "Bananas" shortcut, the search box is empty and the memo list shows the "banana bread" memo (only the shortcut filter applies, so the earlier "apple" search no longer hides it).
