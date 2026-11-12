---
mode: testing
url: http://localhost:15325/auth
headless: true
max_steps: 40
tags: [reenact, issue-4926]
---

# Underscore character stripped out in memo, replaced with italics

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15325/.
Stay on localhost:15325 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor input on the home page
2. Type a memo containing underscores, such as: my_variable_name and other_word_here
3. Click the Save button to create the memo
4. Look at the rendered memo in the memo list
5. Click the edit option from the memo's menu to open edit mode
6. Look at the text in the editor
The reporter says the correct behavior is that the rendered memo shows the underscore characters literally (for example 'my_variable_name' with visible underscores) and no words are shown in italics. Verify that the rendered memo shows the underscore characters literally (for example 'my_variable_name' with visible underscores) and no words are shown in italics.
