﻿// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.
using System;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Documents;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using Microsoft.Azure.Kinect.Sensor;
using Microsoft.Azure.Kinect.Sensor.WPF;

namespace Microsoft.Azure.Kinect.Sensor.Examples.WPFViewer
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        public MainWindow()
        {
            InitializeComponent();
        }

        private async void Window_Loaded(object sender, RoutedEventArgs e)
        {
            using (Device device = Device.Open(0))
            {
                device.StartCameras(new DeviceConfiguration {
                    ColorFormat = ImageFormat.ColorBGRA32,
                    ColorResolution = ColorResolution.r1440p,
                    DepthMode = DepthMode.WFOV_2x2Binned,
                    SynchronizedImagesOnly = true,
                    CameraFPS = FPS.fps30,
                });

                int colorWidth = device.GetCalibration().color_camera_calibration.resolution_width;
                int colorHeight = device.GetCalibration().color_camera_calibration.resolution_height;

                DateTime start = DateTime.Now;
                int frameCount = 0;

                // Allocate image buffers for us to manipulate
                using (Image transformedDepth = new Image(ImageFormat.Depth16, colorWidth, colorHeight))
                using (Image outputColorImage = new Image(ImageFormat.ColorBGRA32, colorWidth, colorHeight))
                using (Transformation transform = device.GetCalibration().CreateTransformation())
                {
                    while (true)
                    {
                        // Wait for a capture on a thread pool thread
                        using (Capture capture = await Task.Run(() => { return device.GetCapture(); }).ConfigureAwait(true))
                        {
                            // Create a BitmapSource for the unmodified color image.
                            // Creating the BitmapSource is slow, so do it asynchronously on another thread
                            Task<BitmapSource> createInputColorBitmapTask = Task.Run(() =>
                            {
                                BitmapSource source = capture.Color.CreateBitmapSource();

                                // Allow the bitmap to move threads
                                source.Freeze();
                                return source;
                            });

                            // Compute the colorized output bitmap on a thread pool thread
                            Task<BitmapSource> createOutputColorBitmapTask = Task.Run(() =>
                            {
                                // Transform the depth image to the perspective of the color camera
                                transform.DepthImageToColorCamera(capture, transformedDepth);

                                // Get Span<T> references to the pixel buffers for fast pixel access.
                                Span<ushort> depthBuffer = transformedDepth.GetPixels<ushort>().Span;
                                Span<BGRA> colorBuffer = capture.Color.GetPixels<BGRA>().Span;
                                Span<BGRA> outputColorBuffer = outputColorImage.GetPixels<BGRA>().Span;

                                // Create an output color image with data from the depth image
                                for (int i = 0; i < colorBuffer.Length; i++)
                                {
                                    // The output image will be the same as the input color image,
                                    // but colorized with Red where there is no depth data, and Green
                                    // where there is depth data at more than 1.5 meters
                                    outputColorBuffer[i] = colorBuffer[i];

                                    if (depthBuffer[i] == 0)
                                    {
                                        outputColorBuffer[i].R = 255;
                                    }
                                    else if (depthBuffer[i] > 1500)
                                    {
                                        outputColorBuffer[i].G = 255;
                                    }
                                }

                                BitmapSource source = outputColorImage.CreateBitmapSource();

                                // Allow the bitmap to move threads
                                source.Freeze();

                                return source;
                            });

                            // Wait for both bitmaps to be ready and assign them.
                            BitmapSource inputColorBitmap = await createInputColorBitmapTask.ConfigureAwait(true);
                            BitmapSource outputColorBitmap = await createOutputColorBitmapTask.ConfigureAwait(true);

                            this.inputColorImageViewPane.Source = inputColorBitmap;
                            this.outputColorImageViewPane.Source = outputColorBitmap;

                            frameCount++;

                            TimeSpan timeSpan = DateTime.Now - start;
                            if (timeSpan > TimeSpan.FromSeconds(2))
                            {
                                double framesPerSecond = (double)frameCount / timeSpan.TotalSeconds;

                                this.fps.Content = $"{framesPerSecond:F2} FPS";

                                frameCount = 0;
                                start = DateTime.Now;
                            }
                        }
                    }
                }
            }
        }
    }
}
