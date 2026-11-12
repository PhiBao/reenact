---
mode: testing
url: http://localhost:15308/auth
headless: true
max_steps: 40
tags: [reenact, issue-5576]
---

# Click shortcut edit button is not pop a edit dialog

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15308/.
Stay on localhost:15308 for the whole run and do not open any other website.
Do these in order:
1. Click the add (+) button next to the Shortcuts section in the sidebar
2. In the create shortcut dialog, type a title for the shortcut
3. Enter a filter for the shortcut if the dialog requires one
4. Click the save/create button to add the shortcut
5. Click the menu (three dots) button of the newly created shortcut item in the sidebar
6. Choose the 'Edit' option from the menu
The reporter says the correct behavior is that after choosing Edit, a dialog titled for editing a shortcut (not creating one) is visible, showing the existing shortcut's title and filter. Verify that after choosing Edit, a dialog titled for editing a shortcut (not creating one) is visible, showing the existing shortcut's title and filter.
