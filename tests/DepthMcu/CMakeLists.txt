# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT License.

add_executable(depthmcu_ft depthmcu_ft.cpp)

target_link_libraries(depthmcu_ft PRIVATE 
    k4ainternal::depth_mcu
    k4ainternal::utcommon
    gtest::gtest
    azure::aziotsharedutil)

k4a_add_tests(TARGET depthmcu_ft HARDWARE_REQUIRED TEST_TYPE FUNCTIONAL)
