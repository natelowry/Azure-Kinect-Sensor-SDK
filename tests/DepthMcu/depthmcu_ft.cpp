// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//************************ Includes *****************************
#include <k4ainternal/depth_mcu.h>
#include <gtest/gtest.h>
#include <utcommon.h>
#include <azure_c_shared_utility/tickcounter.h>

//**************Symbolic Constant Macros (defines)  *************

//************************ Typedefs *****************************

//************ Declarations (Statics and globals) ***************

//******************* Function Prototypes ***********************

//*********************** Functions *****************************

class depthmcu_ft : public ::testing::Test
{
public:
    virtual void SetUp()
    {
    }

    virtual void TearDown()
    {
    }

};

depthmcu_stream_cb_t callback;

void callback(k4a_result_t result, k4a_image_t image, void* context)
{
    (void)result;
    (void)context;
    image_dec_ref(image);
}

TEST_F(depthmcu_ft, test)
{
    depthmcu_t handle;
    depthmcu_firmware_versions_t version;
    ASSERT_EQ(K4A_RESULT_SUCCEEDED, depthmcu_create(0, &handle));

    ASSERT_EQ(K4A_RESULT_SUCCEEDED, depthmcu_get_version(handle, &version));

    printf("Depth build: %d\n", version.depth_build);

    ASSERT_EQ(K4A_RESULT_SUCCEEDED, depthmcu_depth_set_capture_mode(handle, k4a_depth_mode_t::K4A_DEPTH_MODE_PASSIVE_IR));

    ASSERT_EQ(K4A_RESULT_SUCCEEDED, depthmcu_depth_start_streaming(handle, callback, NULL));

    depthmcu_depth_stop_streaming(handle, false);

}

int main(int argc, char **argv)
{
    return k4a_test_common_main(argc, argv);
}
