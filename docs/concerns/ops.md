# Ops

Deployment, configuration, failure and recovery. What it takes to run the thing and to find out
what it is doing.

## Universal

### Configuration

- **Every setting that differs by environment is configuration, not a constant** — `script`
  Grep for hard-coded hosts, keys, paths and limits.
- **Missing configuration fails loudly at start-up** — `test`
  Never silently at the first request, and never with a default that works in development and
  is wrong in production.
- **Configuration is documented where it is defined** — `script`
  An example file, complete, with every key present.

### Deployment

- **A deploy is repeatable from a clean checkout** — `judgment`
- **A deploy can be rolled back** — `judgment`
  And the step that cannot — usually a migration — is known in advance. Cross-reference
  `data.md`.
- **The project runs locally from its documented setup** — `test`
  The instructions in `docs/stack.md` are followed literally on a clean environment, not
  remembered.

### Failure

- **Every external dependency has a defined behaviour when it is down** — `test`
  Fail, degrade, queue, or retry. Name which.
- **Retries are bounded and backed off** — `test`
- **Nothing retries a non-idempotent operation** — `judgment`
- **A failure in a non-essential path does not take down an essential one** — `judgment`

### Observability

- **Errors reach somewhere a person will see them** — `judgment`
  Not only a log file nobody reads.
- **Logs carry enough to trace one request end to end** — `test`
  A correlation identifier.
- **Logs carry no secret and no personal data beyond what is permitted** — `script`
  Cross-reference `security.md`.
- **There is a health check that means something** — `test`
  One that fails when the service cannot do its job, not one that returns 200 while the
  database is unreachable.
- **The things worth alerting on are stated** — `judgment`
  An alert on everything is an alert on nothing.

### Cost

- **Anything billed per use has a bound** — `judgment`
  An unbounded loop against a paid API is a financial incident.

## This project

<!--
Generated at planning, approved by the user, grown by harvest. See README.md.
Record the environments, the deploy mechanism and the alerting destination here on the first
planning sweep.
-->
