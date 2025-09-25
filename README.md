# `cpufreq`

## What

Cpufreq provides a convenient way of setting and getting CPU scaling governors.

Cpufreq is designed according to a modular client server protocol.  `cpufreq` is the user facing command line interface that interacts with the daemon `cpufreqd` through a binary interface specified by `libcpufreq`. One can implement a different client and a different server using `libcpufreq`.
