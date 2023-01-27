# IPC-Actor Client
The client to interact with IPC client.

## Dev
To add a new handler, do the following:
- Create a file for the new handler in `src/cli`.
- Define the parameters required to handle the request by implementing the `CommandLineHandler` trait.
- Register the newly created handler in `cli/mod` using `register_command`.
