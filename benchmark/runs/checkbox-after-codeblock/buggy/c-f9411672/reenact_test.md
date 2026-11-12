---
mode: testing
url: http://localhost:15300/auth
headless: true
max_steps: 40
tags: [reenact, issue-5319]
---

# Wrong checkbox checked when it's preceded by checkbox in a codeblock

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15300/.
Stay on localhost:15300 for the whole run and do not open any other website.
Do these in order:
1. Open the memo editor on the home page to create a new memo
2. Type the line: ```
3. Press Enter and type the line: -[ ] foo
4. Press Enter and type the line: ```
5. Press Enter and type the line: - [ ] bar
6. Click the Save button to save the memo
7. In the saved memo's view mode, click the checkbox next to 'bar'
The reporter says the correct behavior is that The code block still shows '-[ ] foo' unchanged, and the checkbox next to 'bar' is shown as checked. Verify that The code block still shows '-[ ] foo' unchanged, and the checkbox next to 'bar' is shown as checked.
