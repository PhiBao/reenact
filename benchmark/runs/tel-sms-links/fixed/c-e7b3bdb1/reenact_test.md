---
mode: testing
url: http://localhost:15311/auth
headless: true
max_steps: 40
tags: [reenact, issue-6340]
---

# tel: and sms: URI schemes not supported in links.

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15311/.
Stay on localhost:15311 for the whole run and do not open any other website.
Do these in order:
1. Open the memo editor on the home page
2. Type [phone me](tel:+440000000000) into the memo editor
3. Click the Save button to create the memo
4. Look at the rendered memo in the memo list
The reporter says the correct behavior is that the rendered memo shows the text 'phone me' as a clickable link (styled as a hyperlink) that opens a dialer/tel prompt when clicked, rather than as plain unlinked text. Verify that the rendered memo shows the text 'phone me' as a clickable link (styled as a hyperlink) that opens a dialer/tel prompt when clicked, rather than as plain unlinked text.
