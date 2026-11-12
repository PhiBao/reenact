---
mode: testing
url: http://localhost:16203/auth
headless: true
max_steps: 40
tags: [reenact, issue-5236]
---

# Markdown List Auto-Completion Adds Symbol to Current Line Instead of New Line

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:16203/.
Stay on localhost:16203 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click on the memo editor text area
2. Type "- 123"
3. Press the Enter key
The reporter says the correct behavior is that The first line of the editor still reads "- 123" with nothing appended, and a new second line begins with "- " to continue the list. Verify that The first line of the editor still reads "- 123" with nothing appended, and a new second line begins with "- " to continue the list.
