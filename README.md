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

### `order::Order` doesn't use references
This would have likely been obvious to someone experienced, but having references to `layout::Table` and `menu::Item`
caused problems with lifetimes, as it ultimately creates circular references with `&mut self` calls to the repositories.

Luckily, thinking about it, there shouldn't be much cloning in practice if there's some sort of DB/service in the middle,
since data needs to be deserialized into fresh objects anyway.

### Orders store a numeric quantity
At least as the system is currently modeled, there's not necessarily a reason to break each individual item into its own order.
* Ordered items go to tables and are identical, and there's no need to distinguish them.
* Removing >1 items from an order becomes a convenient operation.
* Staff can choose to add items to cut into the queue, or create a new order if more appropriate.
* In a more typical American model, all of the items would be brought out at once.
  The order can be completed/removed all in one go.

The only interesting scenario is the model not uncommon (perhaps actually common) in Japan where items are brought out
as they're done. Having each item in its own order is probably most convenient here.
The current modeling of the service supports this scenario, and clients need only use the service in this way.
Or they can just decrement the quantity as items are done.

That being said, I'm not against ditching `set_quantity`, adding `complete_items` to reduce the quantity,
and only allowing adding items by creating new orders. It's less flexible but easier to reason about, and,
if "cutting in line" is important, implementing an `order_queue` as mentioned above would probably be better anyway.

### `order::Repository` doesn't know about the other `layout` and `menu` repositories
As far as the application is designed, all of these repos are separate. For instance, when asking the order repo for
all of the orders for a table, there's nothing in the design to say if an Err or an empty Vec should be returned.

This isn't much of a design issue, since, if an `order::Repository` implementation needs to be given information about
tables, it can be given it without consideration for the overall design.

Still, it could be argued that having a singular `RestaurantRepository` is the better way to model it. Design is fun!

### API Versioning, extra Path extraction
I tend to prefer versioning via a header, and it was rather obnoxious to hook it up,
though I'm sure there's a better way.

A side-effect of the current employed method is that there's always an extra path extracted
thanks to the initial wildcard. It's something I'll fix in my free time even after submitting the code,
but it's not particularly important for this exercise.

### Miscellaneous
* `anyhow` is currently being used on the repository traits because I haven't been able to find a more effective
  solution to the problem where repositories naturally will have their own custom errors to give.
  It's not a bad solution, since this isn't a library but an application, but, for educational reasons, there's a TODO
  to take another stab at it, time permitting.
* We currently have a hard dependency on Utc::now(). While this won't have an impact for the duration of this exercise,
  it would be preferable, if possible, to consider a "clock" as a driven adapter. Will see if `chrono` has this later on.
* `RepoItem<T>` was introduced, versus each item containing an `id: Some(u32)`,
  to eliminate the awkwardness of figuring out if an item came from a repo or not. Now, it's inherent to the type.
* `The application MUST, upon query request, show a specified item for a specified table number.`...  
  Since I modeled individual orders as having a quantity, versus having an order for each item,
  I believe the call to get all items for a table is sufficient and therefore have not added the above.

## Things I didn't have time for
### Idempotency
To keep things timeboxed, I didn't end up getting to this. If I did get around to it...
* The web service is where this is handled via caching and a client id. 
* Cache is in-memory, which certainly wouldn't work for distributed systems.
* I'm not actually deeply familiar with doing it this way.
  It seems to me that failing (as opposed to not in the cache) to read from or write to the cache would break idempotency.
  Perhaps this is considered acceptable for many cases? For critical cases like those dealing with payments,
  there are other techniques, so I'm pretty okay with this.

Side-note: these kinds of problems are why I like event-sourcing!

### HTTP error results
It's currently plaintext and would ideally be something more structured.

### Regarding `menu::Item::cook_time`
This is likely insufficient. `ordering::Order` is modeled as having a quantity.
Multiples of an item don't necessarily scale linearly, so this model doesn't truly capture the idea of cooking time.

Idea:
* `cook_time` -> `base_cooking_time`
* Add `additional_time`
* Add `max_batch_size` to indicate how many of an item can be cooked at once

Granted, this still doesn't account for multiple tables' orders being cooked at once.
It also doesn't account for orders being queued in a busy restaurant.
Likely, the estimated duration will be calculated by a future `order_queue` component that can use these values effectively.

Additionally, since the client is driven by employees, we can allow them to input a custom estimated duration.

### Always using separate structs on web api-side
It's what I'd normally prefer, really just because it's most obvious what gets sent through the API
(aka no secrets or internal data).
There's a greater than zero percent change that I got around to it and forgot to delete this.