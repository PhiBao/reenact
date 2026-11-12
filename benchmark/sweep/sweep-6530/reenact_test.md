---
mode: testing
url: http://localhost:18090/auth
headless: true
max_steps: 40
tags: [reenact, issue-6530]
---

# Custom URI links are not clickable

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:18090/.
Stay on localhost:18090 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor to start writing a new memo
2. Type the text `obsidian://open?vault=test` into the memo editor
3. Save the memo
4. Look at the saved memo content
The reporter says the correct behavior is that The custom URI `obsidian://open?vault=test` is shown as a clickable link in the memo. Verify that The custom URI `obsidian://open?vault=test` is shown as a clickable link in the memo.
