---
mode: testing
url: http://localhost:16214/auth
headless: true
max_steps: 40
tags: [reenact, issue-5324]
---

# Todo command doesn't work in some desktop browsers

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:16214/.
Stay on localhost:16214 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor on the home page to focus it
2. Make sure the cursor is at the very beginning of an empty line
3. Press the / key
4. Wait for the slash-command menu to appear with todo highlighted
5. Press Enter while the todo option is highlighted
The reporter says the correct behavior is that The editor's empty line shows the text '- [ ] ' after selecting the todo command, and the slash-command menu is closed. Verify that The editor's empty line shows the text '- [ ] ' after selecting the todo command, and the slash-command menu is closed.
