# Frequently Asked Questions

## Encountered a SEGFAULT. What Kind Of Data Types Can Be Transmitted Via iceoryx2

iceoryx2 stores all data in shared memory, which imposes certain restrictions.
Only data that is self-contained and does not use pointers to reference itself
is allowed. This is because shared memory is mapped at different offsets in
each process, rendering absolute pointers invalid. Additionally, if the data
structure uses the heap, it is stored locally within a process and cannot be
accessed by other processes. As a result, data types such as `String`, `Vec`,
or `HashMap` cannot be used as payload types.

Additionally, every data type must be annotated with `#[repr(C)]`. The Rust
compiler may reorder the members of a struct, which can lead to undefined
behavior if another process expects a different ordering.

To address this, iceoryx2 provides shared-memory-compatible data types. You
can refer to the [complex data types example](examples/rust/complex_data_types),
which demonstrates the use of `FixedSizeByteString` and `FixedSizeVec`.

## How To Define Custom Data Types (Rust)

1. Ensure to only use data types suitable for shared memory communication like
   pod-types (plain old data, e.g. `u64`, `f32`, ...) or explicitly
   shared-memory compatible containers like some of the constructs in the
   `iceoryx2-bb-containers`.
2. Add `#[repr(C`)]` to your custom data type so that it has a uniform memory
   representation.

   ```rust
    #[repr(C)]
    struct MyDataType {
        //....
    }
   ```

3. **Do not use pointers, or data types that are not self-contained or use
   pointers for their internal management!**

## How To Define Custom Data Types (C++)

1. Ensure to only use data types suitable for shared memory communication like
   pod-types (plain old data, e.g. `uint64_t`, `int32_t`, ...) or explicitly
   shared-memory compatible containers like some of the constructs in the
   `iceoryx-hoofs`.
2. **Do not use pointers, or data types that are not self-contained or use
   pointers for their internal management!**

## How To Send Data Where The Size Is Unknown At Compilation-Time?

Take a look at the
[publish-subscribe dynamic data size example](examples/rust/publish_subscribe_dynamic_data_size).

The idea is to create a service based on a slice and define at runtime a
`max_slice_len`. Then samples up to a length of the max slice length can be
allocated with `loan_slice{_uninit}`. When it turns out that the slice length is
insufficient, a new publisher with a larger `max_slice_len` can be created.

<!-- markdownlint-disable -->

> [!IMPORTANT]
> Be aware that the history of the old publisher is lost when it is
> removed.

> [!NOTE]
> We are also working on an API that does not require the user to
> explicitly create a new publisher whenever the memory is insufficient. It
> would also solve the history issue.

<!-- markdownlint-enable -->

## How To Make 32-bit and 64-bit iceoryx2 Applications Interoperatable

This is currently not possible since we cannot guarantee to have the same
layout of the data structures in the shared memory. On 32-bit architectures
64-bit POD are aligned to a 4 byte boundary but to a 8 byte boundary on
64-bit architectures. Some additional work is required to make 32-bit and
64-bit applications interoperabel.

## My Transmission Type Is Too Large, Encounter Stack Overflow On Initialization

Take a look at the
[complex data types example](examples/rust/complex_data_types).

In this example the `PlacementDefault` trait is introduced that allows in place
initialization and solves the stack overflow issue when the data type is larger
than the available stack size.

## Does iceoryx2 Offer an Async API?

No, but it is
[on our roadmap](https://github.com/eclipse-iceoryx/iceoryx2/issues/47).
However, we offer an event-based API to implement push notifications. For more
details, see the [event example](examples/rust/event).

## Application does not remove services/ports on shutdown or several application restarts lead to port count exceeded

The structs of iceoryx2 need to be able to cleanup all resources when they go
out of scope. This is not the case when the application is:

* killed with the sigkill signal (`kill -9`)
* the `SIGTERM` signal is not explicitly handled

iceoryx2 already provides a mechanism that registers a signal handler that
handles termination requests gracefully, see
[publish subscribe example](examples/rust/publish_subscribe) and

```rust
while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
  // user code
}
```

But you can also use a crate like [ctrlc](https://docs.rs/ctrlc/latest/ctrlc/).

## How to use `log` or `tracing` as default log backend

* **log**, add the feature flag `logger_log` to the dependency in `Cargo.toml`
    ```toml
    iceoryx2 = { version = "0.1.0", features = ["logger_log"]}
    ```
* **tracing**, add the feature flag `logger_tracing` to the dependency in
  `Cargo.toml`
    ```toml
     iceoryx2 = { version = "0.1.0", features = ["logger_tracing"]}
    ```

## How to set the log level

```rust
use iceoryx2::prelude::*

// ...

set_log_level(LogLevel::Trace);
```

## A crash leads to the failure `PublishSubscribeOpenError(UnableToOpenDynamicServiceInformation)`

**Note:** A command line tool and internal service is already planned to cleanup
resources from crashed applications, see issue #65.

When an application crashes some resources may remain in the system and must be
cleaned up manually. If this occurs, stop all services and remove manually all
shared memory segments and static service config files.

```sh
rm -rf /dev/shm/iox2_*
rm -rf /tmp/iceoryx2/*
```

If you cannot stop all running services, you can look up the `uuid` of the
service in question and remove the files manually. Assume, the service
`My/Funk/ServiceName` is corrupted. You can identify the static config by
grepping the service name in the `/tmp/iceoryx2/service` folder.

So the command

```sh
cd /tmp/iceoryx2/service
grep -RIne "My/Funk/ServiceName"
```

provides us with the output

```text
iox2_25b25afeb7557886e9f69408151e018e268e5917.service:2:service_name = "My/Funk/ServiceName"
```

The file name corresponds with the `uuid` of the service. So removing the
dynamic and static service config with the following commands, removes the
service completely from the system.

```sh
# static service config
rm /tmp/iceoryx2/service/iox2_25b25afeb7557886e9f69408151e018e268e5917.service

# dynamic service config
rm /dev/shm/iox2_25b25afeb7557886e9f69408151e018e268e5917.dynamic
```

Be aware, that if an application with a publisher crashes, the data segment of
the publisher must be cleaned up as well.
