# Security

Authentication, authorisation, input, secrets, exposure. The dimension where the failure mode is
silent until it is catastrophic, which is why the entries lean toward `test` over inspection.

## Universal

### Authorisation

- **Every endpoint and action checks authorisation server-side** — `test`
  A hidden button is not an authorisation check.
- **Authorisation is checked on the object, not only the route** — `test`
  The classic defect: a user may reach `/orders/:id` and sees any order. Test with a valid
  session and another user's identifier.
- **Enumeration does not leak existence** — `test`
  "Not found" and "not yours" are the same response where the distinction is itself
  information.
- **Privilege cannot be escalated through a mass-assigned field** — `test`
  A role, an owner or a flag accepted from request input.

### Input

- **Every input is validated server-side** — `test`
  Client validation is a convenience.
- **Queries are parameterised** — `script`
  Grep for string-built queries.
- **Output is escaped for its context** — `test`
  HTML, attribute, URL, SQL, shell are different contexts.
- **Uploads are constrained by type, size and count, and stored outside the served root** — `test`
- **Redirect targets are validated against an allow-list** — `test`

### Identity and session

- **Credentials are hashed with a current algorithm** — `script`
- **Sessions are invalidated on logout and on password change** — `test`
- **State-changing requests carry a CSRF defence** — `test`
- **Authentication and anything brute-forceable is rate-limited** — `test`
  Including anything with a guessable short code.

### Secrets and exposure

- **No secret is in the repository** — `script`
  Scanned in CI, not reviewed by eye.
- **No secret reaches a log, an error page, an analytics event, or the client bundle** — `script`
- **Errors shown to users carry no internal detail** — `test`
  Stack traces, query text, paths, versions.
- **Dependencies are scanned, and findings are triaged rather than accumulated** — `script`

### Data in transit and at rest

- **Transport is encrypted everywhere, including between internal services** — `judgment`
- **What is encrypted at rest, and what is not, is a recorded decision** — `judgment`

## This project

<!--
Generated at planning, approved by the user, grown by harvest. See README.md.
Record the threat model, the roles, and anything exempt here on the first planning sweep.
-->
