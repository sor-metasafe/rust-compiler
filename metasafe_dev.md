## Relevant Notes:
### Analysis:
# Goal:
Try using local shadows. We will compare the cost of SFI
Problem with SFI is memory allocation and shadowing every pointer seems expensive.
Currently MetaSafe computes the address of every shadow instead of getting one pointer at a time.
The challenge here will be whether we can avoid this.

# The Plan:
If a certain inhouse object is accessed for a write,
write to a shadow. From the first write in a given function scope, use the shadow, including all reads succeeding the first write. At the end of the function, if the object is still alive, then:
- Enable MPK
- Write everything from the shadow back to the object.
- Disable MPK.
The question at this point becomes whether we can enable and disable MPK safely.

# Further optimizations?
The assumption is that all such inhouse objects are on the heap. If we can confirm that such an object is stack-based, then
MPK need not be enabled. As it will reside on the safe stack anyway.
This analysis will only be done locally. We can't really afford inter-procedural PTA, as we saw how it affects compile time in TRust.


## Dev notes:
# Analysis: Relevant places:
- Rustc borrow:
    > Liveness (rustc_borrow)


## Integration with TRust:
To integrate MetaSafe with TRust, we need to see the following are accomplished:
    > External functions run on an a separate stack. For this, we wrap external functions like the stacker crate does. Except that, different from the stacker crate, we don't want to mmap a new stack every time an external fuction is called, rather, we will mmap an initial stack at once, set the libc::MAP_STACK option, and reuse this stack forever. Of course, once the function returns, we would have to create another stack. 
    > Create two more stacks; one for the smart pointers, the other for unsafe objects from TRust. Following this requirement, we hook the pthread function as in TRust and SafeStack to ensure these stacks always exist.
    > Perform pointer-to-analysis (PTA) to isolate unsafe objects from safe ones. TRust uses SVF, but we will avoid that and run MIR analysis. This is all work btw! I'm not sure Prof. Moon will approve as this sounds like too much work for a major revision, but we have to do it.
