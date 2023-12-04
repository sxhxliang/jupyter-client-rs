# Adding a new SHELL message

These are the steps to add a new Command/Response pair:

- Add a new enum variant for `Command`
- Add a new implementation to turn the command into a wire message (`Command::into_wire`)
- Add a new enum variant for `Response`
- Add a new deserializable struct for response content
- Add an entry to match the `msg_type` in `WireMessage::into_response`
- Add `into_packets` tests in `wire.rs`
- Add `message_parsing` tests in `responses.rs`

# Messages requiring implementation

## Client -> Kernel (SHELL)

- [x] `kernel_info_request`
- [x] `execute_request`
- [x] `inspect_request`
- [x] `complete_request`
- [x] `history_request`
- [x] `is_complete_request`
- [x] `shutdown_request`
- [x] `comm_info_request`
- [x] `async execute code`
- [ ] `interrupt_request`

## Kernel -> Client (SHELL)

- [x] `kernel_info_reply`
- [x] `execute_reply`
- [x] `inspect_reply`
- [x] `complete_reply`
- [x] `history_reply`
- [x] `is_complete_reply`
- [x] `shutdown_reply`
- [x] `comm_info_reply`

## Kernel -> Client (IOPUB)

- [x] `stream`
- [x] `status`
- [x] `execute_input`
- [x] `error`
- [x] `execute_result`
- [x] `clear_output`
- [x] `async result`
- [ ] `display_data`
- [ ] `update_display_data`

## Kernel -> Client (STDIN)

- [ ] `input_request`

## Client -> Kernel (STDIN)

- [ ] `input_reply`

## Kernel -> Client (CONTROL)

- [ ] `interrupt_reply`
