---
mode: testing
url: http://localhost:18090/auth
headless: true
max_steps: 40
tags: [reenact, issue-6550]
---

# Numbered lists >10 overflow to the next line

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:18090/.
Stay on localhost:18090 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor on the home page to focus it
2. Type a numbered list with at least 10 items, one per line: '1. one', '2. two', '3. three', '4. four', '5. five', '6. six', '7. seven', '8. eight', '9. nine', '10. ten', '11. eleven'
3. Click the Save button to post the memo
4. Look at the rendered numbered list in the posted memo, specifically items 10 and 11
The reporter says the correct behavior is that In the rendered memo, the numbered list items 10 and 11 each show their number marker and their text on the same line, and the list does not look broken. Verify that In the rendered memo, the numbered list items 10 and 11 each show their number marker and their text on the same line, and the list does not look broken.
