---
mode: testing
url: http://localhost:15309/auth
headless: true
max_steps: 40
tags: [reenact, issue-5576]
---

# Click shortcut edit button is not pop a edit dialog

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15309/.
Stay on localhost:15309 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the add (+) button in the Shortcuts section of the sidebar
2. Enter a title for the new shortcut and a filter if the dialog requires one
3. Click the confirm/create button to save the shortcut
4. Click the menu (three-dot) button next to the new shortcut item in the sidebar
5. Click the 'Edit' option in the menu
The reporter says the correct behavior is that After clicking Edit, a dialog titled for editing a shortcut (e.g. 'Edit Shortcut') is shown, pre-filled with the existing shortcut's values, rather than a create-shortcut dialog. Verify that After clicking Edit, a dialog titled for editing a shortcut (e.g. 'Edit Shortcut') is shown, pre-filled with the existing shortcut's values, rather than a create-shortcut dialog.
