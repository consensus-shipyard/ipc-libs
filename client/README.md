# IPC-Actor Client
The client to interact with IPC client.

## Dev
To add a new handler, do the following:
- Create a file for the new handler in `src/command`.
- Define the parameters required to handle the request by implementing the `CommandLineHandler` and `RPCNodeHandler` trait.
- Register the newly created handler in `main` using `register_cli_command` and `register_server_routes`.
