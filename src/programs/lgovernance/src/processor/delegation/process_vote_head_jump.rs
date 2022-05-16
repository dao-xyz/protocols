/// When undelegating we have to go through each casted vote of the delegatee
/// This is unecessary for inactive (old votes)
/// This process is intended to skip iteration of old/irrelevant votes