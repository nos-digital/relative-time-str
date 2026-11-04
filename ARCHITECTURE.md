# Architecture
Each of the Platform's repository contains the code for a single microservice,
which in turn is focused on a single specific task. Most of these repositories
adhere to a standardized code-flow and module structure, which has been dubbed
"Enterprise Architecture".  This documents provides a high-level overview of
the architecture of this and aims to give the reader some guidance in what
to look for, where to look for it, and why it was done this way.


## 1. Project Structure
This section provides a high-level overview of the project's directory and
file structure, categorised by architectural layer or major functional area.
It is essential for quickly navigating the codebase, locating relevant files,
and understanding the overall organization and separation of concerns.

```
[Project Root]/
├── .git/                     # Files belonging to git for version controll
├── .cargo/                   # Cargo linting configuration
├── migrations                # SQL files for the database definition
├── src/                      # Contains most of the source code
│   ├── config/               # Configuration type definitions
│   ├── v[n]/                 # Source for a specific API version
│   │   ├── controller/       # Endpoint controllers containing most of the business logic
│   │   ├── dto/              # Output type definitions and transformations from models
│   │   ├── model/            # Input type definitions (external APIs or databases)
│   │   ├── repository/       # Controller-specific traits returning models
│   │   ├── repository_impls/ # Implementations of the Controller-specific repository traits
│   │   ├── test/             # Controller integration tests
│   │   ├── mod.rs            # Module listing all sub-modules
│   │   ├── openapi.json      # OpenAPI schema of this API version
│   │   ├── openapi.md        # OpenAPI documentation text
│   │   └── routes.rs         # Router and route definitions
│   ├── error.rs              # Error handling
│   ├── main.rs               # Server starting and setup with additional non-API routes
│   ├── state.rs              # State definition used across routes
│   ├── test.rs               # Utility code for testing
│   └── utils.rs              # Utility code
├── .env                      # Environment variables for the configuration, ignored by git
├── .env.example              # Example environment variables for the configuration
├── .github/                  # GitHub Actions CI (calls nos-github-actions' rust-ci workflow)
├── .gitignore                # Specifies intentionally untracked files to ignore
├── ARCHITECTURE.md           # This document
├── build.rs                  # Pre-build triggers, such as setting the git SHA
├── Cargo.lock                # Dependency versions the project was last successfully build with
├── Cargo.toml                # Project package definition and packages
├── CLAUDE.md                 # Documentation and guidance for Claude
├── clippy.toml               # Additional clippy configuration
├── CODEOWNERS                # Github repository ownership configuration
├── deny.toml                 # Settings for cargo-deny
├── format_unstable.sh        # A helper script to format the source code with additional settings
├── README.md                 # Project overview and quick start guide
├── runtime.Dockerfile        # Runtime image for GitHub Actions (CI adds the binaries)
├── rustfmt.toml              # Formatting settings used by rustfmt
└── rustfmt-unstable.toml     # Unstable formatting settings used by rustfmt
```

> [!NOTE]
> In some cases a microservice does not have versioned API endpoints, in
> that case the version directory should be omitted and its sub-directiess
> moved up one directory.


## 2. More Project Structure
In addition to the structure outlined in section 1, we have some additional
project structure and conventions.

1. Prefer `mod.rs` files over `name/` and `name.rs` files. This way the root
   of the module remains close with the files in the module itself.

2. `mod.rs` files contain mostly only `mod sub_module;` and `pub use
   sub_module::*;` statements, making it easier to find code based on
   file name, whilst also providing callside an easy module API (e.g. `use
   dto::MyType;` instead of `use dto::my_type::MyType;`).

3. Each type in the `dto` and `model` directories should have its own file,
   making it easier to find code based on file name.

4. `controllers` should be one function per file and should not contain
   `axum` specific logic.

5. `repository` traits should be one trait per file and should contain only
   one function per trait.

5. `test` should contain one directories per controller being tested, where
   the snapshots of the tests are placed within the module.

6. `model`s, `dto`s, `repository`s, and `controllers` should not be shared
   between API versions.


## 3. Used Tools and Libraries
Outside of the direct source, most services use the same libraries and
tooling (outside of the most commonly known ones like `axum` and `serde`).
Which tools and libraries are outlined here.

### Webservice SDK
This libraries contains a bunch of code to enable easy implementation of a
microservice and reduce boilerplate. There are various sub-libraries each
containing the wrappers and functions for a specific use-case. It's preferred
to use and improve code exposed through there than having a duplicate or
alternative version of functionallity provided there. This way each other
microservice can benefit from the changes and each microservice becomes more
standardized.

### Corgi SDK
Our client-facing microservices should obtain their data from Corgi. To
facilitate this, similar to the Webservice SDK, the Corgi codebase contains
its own SDK. This SDK contains a `DeprecationTest` macro, which scans the
query for any fields marked as deprecated. To allow for easier upgrading
the query schema, this macro should always be used on query structs.

### SQL with `sqlx` and `sleek`
For SQL we use the `sqlx` crate. `sqlx` allows for compile time checking of
queries, which is preferred. The intermediate files for these queries is
stored in the `.sqlx` directory located in the root of the project structure
and should be checked in into version controll. CI can also check these
files for the correctness of the SQL statements compared to the database
schema.  In addition, the migrations should be formatted. This can be done
with the [sleek CLI tool](https://crates.io/crates/sleek).

### Testing with `insta`
For testing we use unit tests where possible, however for controllers we use
snapshot testing. These tests and their snapshots are placed in the `test`
directory. We aim for each snapshot to capture one part of the business logic.

### Checking with `cargo-deny`
To ensure no vulnerabilities creep into our codebases we use `cargo-deny`
in CI together with the checked-in `Cargo.lock` to check for any CVEs (and
some other settings).

