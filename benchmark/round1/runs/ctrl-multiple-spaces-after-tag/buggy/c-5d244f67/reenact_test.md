---
mode: testing
url: http://localhost:15326/auth
headless: true
max_steps: 40
tags: [reenact, issue-5382]
---

# 标签与内容之间有多个空格时，只保留了一个空格

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15326/.
Stay on localhost:15326 for the whole run and do not open any other website.
Do these in order:
1. Click the memo editor input box on the home page
2. Type "#tag content" (one space between the tag and the word)
3. Click the Save/Publish button to publish the memo
4. Click the memo editor input box again
5. Type "#tag" followed by 24 space characters and then "content"
6. Click the Save/Publish button to publish the memo
7. Look at how the second published memo is rendered in the memo list, comparing the gap between "#tag" and "content" with the first memo
The reporter says the correct behavior is that In the second published memo, "content" appears after "#tag" with a visibly wider gap than in the first memo, so the multiple spaces typed are preserved. Verify that In the second published memo, "content" appears after "#tag" with a visibly wider gap than in the first memo, so the multiple spaces typed are preserved.
