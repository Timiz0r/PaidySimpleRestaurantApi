# PaidySimpleRestaurantApi
As described [here](https://github.com/paidy/interview/blob/master/SimpleRestaurantApi.md).

## Design methodology
### Ports-and-adapters
The system is designed around the ports-and-adapters pattern.
As a quick overview of ports-and-adapters:
* An application's boundaries -- to which they hook up to other applications -- are bound by its ports.
* A driver port calls into the application and is typically concrete implementation.
* A driven port is how the application makes calls outside the application and is typically an abstract interface.
* A driver adapter is what interfaces with the concrete diver port.
* A driven adapter is the concrete implementation of the interface defined by the driven port.
* The inner logic of the application is completely irrelevant to ports-and-adapters.
* The adapters can be their own, other "ports-and-adapters-style" applications if of sufficient size,
  but not necessarily so.

### Automated testing
Ports-and-adapters allow for an uncommon redefinition of automated testing.
* Unit tests, instead of testing individual classes (and often all of them), test through the port boundaries. Advantages include...
  * Less tests with just as much coverage
  * Testing the code the way it is to be used in practice
  * Changes to the application's inner implementation don't require changing unit tests
    * Refactoring and redesign of internal implementation doesn't involve changing unit tests for maximum safety
    * Refactoring and redesign is less time-consuming,
      since unit tests aren't creating tight dependencies with the implementation details of the application.
  * Testing each class often leads to unnecessary amounts of abstraction to support dependency injection.
* Integration tests test how two or more applications integrate with each other
  * Testing is still performed via port boundaries
* System tests include disk, network (HTTP, etc.), database, etc.
* Depending on the design, it can be more convenient to lump integration and system testing together.
* Testing individual classes can still have its place, but it of course has a cost.

## Design notes
### High-level design
For this system, the setup looks like this:
* The `restaurant` application manages orders.
* Its driver ports handle...
  * Orders -- creation, completion, cancellation, etc.
  * These ports have been left unimplemented but are included to better illustrate
    what a more complete design may look like.
    * Defining what tables and seats there are.
    * What the menu is, including how long it takes to prepare items.
* Its driven ports are interfaces used to store data
* Its driver adapters are...
  * A web API that calls into the relevant driver ports
  * Unit test cases (and potentially simulators) -- more on this in a bit
* Its driven adapters are...
  * An implementation of the driven port that uses SQLite
  * Mocks and/or simulators for unit testing

### Regarding `menu::Item::cook_time`
This is likely insufficient. `ordering::Order` is modeled as having a quantity.
Multiples of an item don't necessarily scale linearly, so this model doesn't truly capture the idea of cooking time.

Idea:
* `cook_time` -> `base_cooking_time`
* Add `additional_time`
* Add `max_batch_size` to indicate how many of an item can be cooked at once

Granted, this still doesn't account for multiple tables' orders being cooked at once.
It also doesn't account for orders being queued in a busy restaurant.
Likely, the estimated time will be calculated by a future `order_queue` component that can use these values effectively.

### `order::Order` doesn't use references
This would have likely been obvious to someone experienced, but having references to `layout::Table` and `menu::Item`
caused problems with lifetimes, as it ultimately creates circular references with `&mut self` calls to the repositories.

Luckily, thinking about it, there shouldn't be much cloning in practice if there's some sort of DB/service in the middle,
since data needs to be deserialized into fresh objects anyway.

### Miscellaneous
* `anyhow` is currently being used on the repository traits because I haven't been able to find a more effective
  solution to the problem where repositories naturally will have their own custom errors to give.
  It's not a bad solution, since this isn't a library but an application, but, for educational reasons, there's a TODO
  to take another stab at it, time permitting.
* We currently have a hard dependency on Utc::now(). While this won't have an impact for the duration of this project,
  it would be preferable, if possible, to consider a "clock" as a driven adapter. Will see if `chrono` has this later on.
* `RepoItem<T>` was introduced, versus each item containing an `id: Some(u32)`,
  to eliminate the awkwardness of figuring out if an item came from a repo or not. Now, it's inherent to the type.