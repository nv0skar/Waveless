# Introduction
## What is Waveless?
*Waveless* is a framework for building APIs but reimagined from the ground up: instead of manually declaring your endpoints in some programming language (which might not be the most performant nor the most optimized code) and handling all their logic (or asking a LLM to do it) in *Waveless* you define your endpoints with config files, where you can control all its behaviour.
> You might have certain endpoints that query database A and require a logged in user with a certain role, while others query database B and do not require any authentication whatsoever.

You might use an arbitrary number of database instances (as many as your service requires), the same rule applies with authentication mechanisms.

Simply ask any LLM to build a web service with Waveless which is fast and safe by default instead of letting it write a full-blown project from scratch (prone to errors and not always the most performant).

## Reasoning about the compiler-executor architecture
A *Waveless* project consists of a main `config.toml` and an `endpoints` folder where all the endpoint definitions are located, you might have multiple endpoints files each defining multiple endpoints at once. When you create a new project these files are automatically generated with sensible defaults. 
From this point, you might have two options: running the server (we're calling 'executor' to the server) or 'building' the project and generate a *Waveless'* build (which will be later 'executed' by the 'executor').
But, why not just running the server and avoid intermediate files? Well, this is done to tackle a variety of issues:
- *Waveless* can automatically generate endpoints from existing schemas such as database tables, you may end up serving endpoints in production you never intended to serve.
- If the database's schema changes on production and a *Waveless'* instance is spawned, it will refuse to start.
- Making lighter builds with only the runtime making it suitable for serverless functions or WASM runtimes (where not all the *Waveless'* features might be needed).

## Discussing the extensibility
To make endpoint definition as generic as possible *Waveless* offers mechanisms to extend the the core functionalty. Currently you can add:
- Database connectors (currently only MySQL compatible databases are implemented, any other SQL based database integration is trivial and might be available in the future).
- Database schema definitions: describes how to generate endpoints from any database. From simple per table API generation to complex LLM pipelines (soon...)
- Request handlers: handles the request, currently only a MySQL query executor is implemented, in the future simple scripting will be allowed, currently you could write custom request handlers by implementing the `AnyExecute` trait and loading it into the binary (more on that later).
- Authentication mechanisms: we currently have a simple role based email-password authentication (with session tokens) (implemented on the MySQL database connector).

### Loading custom components
Loading extensions into *Waveless* is an interesting topic, where you might want to find the perfect balance of speed and complexity. We have proposed the following approaches:
1. Dynamic library loading: the least secure by far yet the easiest, you compromise memory safety, where a malicious component or a vulnerable one might compromise the main runtime's memory or crashing the whole application.
2. IPC mechanisms: instead of loading foreign code, *Waveless'* foreign components implement an interface for IPC, where the main runtime spawns worker for each component, even internal ones.
    - This is useful in situations where a request handler might crash with malformed requests, but as it's running as a worker, the main runtime is still running which respawns the crashed worker.
3. Using *Waveless* as a library: brings the *Waveless* executor to your application, define all the *Waveless'* components directly in your application and integrate an API with your code, without setting up complex authentication, database connection management or routing logic.
