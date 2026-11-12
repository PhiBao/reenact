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
Verify the user is signed in and the main memo page is visible.

## Reproduce the report
Open http://localhost:15301/.
Stay on localhost:15301 for the whole run and do not open any other website.
Before the main steps, make sure of this: a memo exists whose content is a fenced code block containing the line "-[ ] foo" followed by a task list item "- [ ] bar" after the code block.
Do these in order:
1. Open the new memo editor on the home page
2. Type the line ``` (opening code fence)
3. Press Enter and type "-[ ] foo"
4. Press Enter and type ``` (closing code fence)
5. Press Enter and type "- [ ] bar"
6. Click the Save button to create the memo
7. In the saved memo in view mode, click the checkbox next to "bar"
The reporter says the correct behavior is that the checkbox next to "bar" is checked, and the code block still shows "-[ ] foo" unchanged. Verify that the checkbox next to "bar" is checked, and the code block still shows "-[ ] foo" unchanged.
