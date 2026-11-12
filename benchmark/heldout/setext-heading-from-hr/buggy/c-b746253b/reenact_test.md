---
mode: testing
url: http://localhost:16200/auth
headless: true
max_steps: 40
tags: [reenact, issue-5314]
---

# Inconsistent markdown rendering

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:16200/.
Stay on localhost:16200 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor input box on the home page
2. Type "Top"
3. Press Enter
4. Type "--- " (three hyphens followed by a space)
5. Press Enter
6. Type "**Bolded** words"
7. Press Enter
8. Type "---"
9. Press Enter
10. Type "**more, same bolded** regular text"
11. Click the Save button to save the memo
The reporter says the correct behavior is that In the saved memo, "Top" and "Bolded words" appear as regular body-sized text (not as large heading text), with "Bolded" in bold, and "more, same bolded" in bold followed by "regular text". Verify that In the saved memo, "Top" and "Bolded words" appear as regular body-sized text (not as large heading text), with "Bolded" in bold, and "more, same bolded" in bold followed by "regular text".
