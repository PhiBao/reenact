---
mode: testing
url: http://localhost:15301/auth
headless: true
max_steps: 40
tags: [reenact, issue-5319]
---

# Wrong checkbox checked when it's preceded by checkbox in a codeblock

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15301/.
Stay on localhost:15301 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Open the memo editor to create a new memo
2. Type a line containing three backticks (```) to open a fenced code block
3. Press Shift+Enter to add a new line and type '-[ ] foo'
4. Press Shift+Enter to add a new line and type three backticks (```) to close the code block
5. Press Shift+Enter to add a new line and type '- [ ] bar'
6. Click the Save button to save the memo
7. In the saved memo's view mode, click the checkbox next to 'bar'
The reporter says the correct behavior is that After clicking, the checkbox next to 'bar' is checked, and the code block still shows the unchecked text '-[ ] foo'. Verify that After clicking, the checkbox next to 'bar' is checked, and the code block still shows the unchecked text '-[ ] foo'.
