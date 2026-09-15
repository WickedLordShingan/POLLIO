# MINI ASYNC ENGINE

## SPLIT OF ROLES
- ### EXECUTOR
  Executor, as the name suggests executes the tasks. It has a taskqueue contains the ready tasks. If a task returns pending then it is yeeted out of the queue and it's
  waker is registered in the reactor. When the queue is empty the executor thread goes to sleep (*parks itself*).
- ### REACTOR
  Reactor, in our case, is analagous to the interrupt handler. It watches fd's for events and calls the respective wakers. The wakers attach the task back to the executors
  task queue and wake up the executor thread (*unpark*)

### THIS SAVES A LOT OF CPU !! 
![LOW_CPU_USAGE][low_cpu.png]
