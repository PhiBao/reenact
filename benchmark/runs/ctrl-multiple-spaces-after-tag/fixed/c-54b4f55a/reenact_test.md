---
mode: testing
url: http://localhost:15327/auth
headless: true
max_steps: 40
tags: [reenact, issue-5382]
---

# 标签与内容之间有多个空格时，只保留了一个空格

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15327/.
Stay on localhost:15327 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor input on the home page
2. Type "#tag content" into the editor
3. Click the Save/Publish button to post the memo
4. Click the memo editor input again
5. Type "#tag content" (the text #tag, then 24 spaces, then content) into the editor
6. Click the Save/Publish button to post the memo
The reporter says the correct behavior is that The second memo renders "#tag" and "content" with the multiple spaces between them still visible, so the gap is wider than in the first memo's "#tag content". Verify that The second memo renders "#tag" and "content" with the multiple spaces between them still visible, so the gap is wider than in the first memo's "#tag content".
