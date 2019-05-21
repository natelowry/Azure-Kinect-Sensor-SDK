# Release Process

The release process and building of private binaries is managed by Microsoft Azure Kinect team members.

Released builds are numbered using semantic versioning. The GitHub project and CI builds are not uniquely versioned. 
Versioning is applied by the private Azure Kinect Packing build system.

The packaging build system may produce multiple builds with the same build number e.g. "1.0.1-beta.0", however only 
one build of that number will ever be distributed internally or externally for validation or release.

Releases are scheduled on demand based on program needs.

## Release Types

### Alpha Release

Alpha builds are built using source from the develop branch and are numbered with the
expected release number for current development, such as ```1.1.0-alpha.0```.

Alpha builds expect heavy churn and are not guaranteed to be backward compatible with each other.

### Beta Release

Beta builds are built from the release branches, such as ```1.1.0-beta.0```.

Release branches are created when that release is being stabilized, at which point only bug fixes and changes 
required for that release are merged or cherry-picked in to the release branch. Fixes may alternatively be made 
to the release branch directly and then merged back to the develop branch.

### Official Release

Once a beta build has been signed off for release, an official build is created with code from that release branch,
such as ```1.1.0```

### Patch releases

Critical changes to a released build may be made in the release branch to patch an existing release. These
changes do not introduce functionality or break compatibility.

Changes are made in the release branch for the existing release, such as ```release/1.0.x```, and are verified with beta
builds for the patch, such as ```1.0.1-beta.0```, before the patch is signed off and released as ```1.0.1```

## Moving changes between release branches

When a release branch is created it should be created from the develop branch.

Changes may be merged (not squashed) from develop in to a release branch so long as there are no new
changes in develop not suitable for that release.

Once develop starts taking changes for the next release, changes must be cherry-picked or made
directly in a release branch.

Release branches should always be merged back in to develop (not squashed) after changes have been made
there to avoid future merge conflicts.

Our Github repository policy enforces that all pull requests are squashed. Therefore merges between
release branches should be done locally, and the results pushed by a repository administrator.

## Building a Release Package

The Azure Kinect team will update the packaging build repository to reference the commit of the GitHub
Azure Kinect Sensor SDK repository to be released.
The Azure Kinect team will schedule an official build of the packaging repository with the correct
release version number.

The build will produce an official copy of the SDK with the depth engine and installer. All binaries
are signed and the symbols are indexed.

Once a candidate build has been produced, it can be submitted to the Azure Kinect Release pipeline.

The pipeline will:

* Run all release tests.
* Request manual sign off validation.
* Update the documentation resources.
* Tag commit in the Azure Kinect Sensor SDK repository.
* Publish NuGet feeds.
* Request publishing of the MSI installer to the web.

## Installer

The Azure Kinect SDK installer provides both open and closed source binaries. The open source binaries are built from
this [GitHub repo](https://github.com/Microsoft/Azure-Kinect-Sensor-SDK).

[Release tags](https://github.com/Microsoft/Azure-Kinect-Sensor-SDK/releases) are used by Microsoft to label commits
used to create a released build. Checkout the commit that matches the release label to build the desired release version.

## Artifact and Packaging
### Windows MSI
File | Source | Package(s)
---- | ------ | ----------
LICENSE.txt | Microsoft Internal | MSI
REDIST.txt | Microsoft Internal | MSI
ThirdPartyNotices.txt | Microsoft Internal | MSI
version.txt | Microsoft Internal | MSI
sdk\include\k4a\k4a.h | GitHub Source | MSI
sdk\include\k4a\k4a_export.h | GitHub Source | MSI
sdk\include\k4a\k4atypes.h | GitHub Source | MSI
sdk\include\k4a\k4aversion.h | GitHub Source | MSI
sdk\include\k4arecord\k4arecord_export.h | GitHub Source | MSI
sdk\include\k4arecord\playback.h | GitHub Source | MSI
sdk\include\k4arecord\record.h | GitHub Source | MSI
sdk\include\k4arecord\types.h | GitHub Source | MSI
sdk\netstandard2.0\build\Microsoft.Azure.Kinect.Sensor.targets | Microsoft Internal | MSI
sdk\netstandard2.0\release\Microsoft.AzureKinect.deps.json | GitHub Build | MSI
sdk\netstandard2.0\release\Microsoft.AzureKinect.dll | GitHub Build | MSI
sdk\netstandard2.0\release\Microsoft.AzureKinect.pdb | GitHub Build | MSI
sdk\netstandard2.0\release\Microsoft.AzureKinect.xml | GitHub Build | MSI
sdk\windows-desktop\bin\amd64\release\depthengine_1_0.dll | Microsoft Internal | MSI
sdk\windows-desktop\bin\amd64\release\k4a.dll | GitHub Build | MSI
sdk\windows-desktop\bin\amd64\release\k4a.pdb | GitHub Build | MSI
sdk\windows-desktop\bin\amd64\release\k4arecord.dll | GitHub Build | MSI
sdk\windows-desktop\bin\amd64\release\k4arecord.pdb | GitHub Build | MSI
sdk\windows-desktop\build\Microsoft.Azure.Kinect.Sensor.targets | Microsoft Internal | MSI
sdk\windows-desktop\lib\amd64\release\k4a.lib | GitHub Build | MSI
sdk\windows-desktop\lib\amd64\release\k4arecord.lib | GitHub Build | MSI
sdk\windows-desktop\lib\amd64\release\cmake\k4a\k4aConfig.cmake | GitHub Build | MSI
sdk\windows-desktop\lib\amd64\release\cmake\k4a\k4aConfigVersion.cmake | GitHub Build | MSI
sdk\windows-desktop\lib\amd64\release\cmake\k4a\k4aTargets-relwithdebinfo.cmake | GitHub Build | MSI
sdk\windows-desktop\lib\amd64\release\cmake\k4a\k4aTargets.cmake | GitHub Build | MSI
sdk\windows-desktop\lib\amd64\release\cmake\k4arecord\k4arecordConfig.cmake | GitHub Build | MSI
sdk\windows-desktop\lib\amd64\release\cmake\k4arecord\k4arecordConfigVersion.cmake | GitHub Build | MSI
sdk\windows-desktop\lib\amd64\release\cmake\k4arecord\k4arecordTargets-relwithdebinfo.cmake | GitHub Build | MSI
sdk\windows-desktop\lib\amd64\release\cmake\k4arecord\k4arecordTargets.cmake | GitHub Build | MSI
tools\AzureKinectFirmwareTool.exe | GitHub Build | MSI
tools\AzureKinectFirmwareTool.pdb | GitHub Build | MSI
tools\depthengine_1_0.dll | Microsoft Internal | MSI
tools\k4a.dll | GitHub Build | MSI
tools\k4a.pdb | GitHub Build | MSI
tools\k4arecord.dll | GitHub Build | MSI
tools\k4arecord.pdb | GitHub Build | MSI
tools\k4arecorder.exe | GitHub Build | MSI
tools\k4arecorder.pdb | GitHub Build | MSI
tools\k4aviewer.exe | GitHub Build | MSI
tools\k4aviewer.pdb | GitHub Build | MSI

### Windows NuGet
File | Source | Package(s)
---- | ------ | ----------
LICENSE.txt | Microsoft Internal | NuGet
REDIST.txt | Microsoft Internal | NuGet
ThirdPartyNotices.txt | Microsoft Internal | NuGet
version.txt | Microsoft Internal | NuGet
build\native\Microsoft.Azure.Kinect.Sensor.targets | NuGet
build\native\include\k4a\k4a.h | GitHub Source | NuGet
build\native\include\k4a\k4atypes.h | GitHub Source | NuGet
build\native\include\k4a\k4aversion.h | GitHub Source | NuGet
build\native\include\k4a\k4a_export.h | GitHub Source | NuGet
build\native\include\k4arecord\k4arecord_export.h | GitHub Source | NuGet
build\native\include\k4arecord\playback.h | GitHub Source | NuGet
build\native\include\k4arecord\record.h | GitHub Source | NuGet
build\native\include\k4arecord\types.h | GitHub Source | NuGet
build\netstandard2.0\Microsoft.Azure.Kinect.Sensor.targets | NuGet
lib\native\amd64\release\depthengine_1_0.dll | Microsoft Internal | NuGet
lib\native\amd64\release\k4a.dll | GitHub Build | NuGet
lib\native\amd64\release\k4a.lib | GitHub Build | NuGet
lib\native\amd64\release\k4a.pdb | GitHub Build | NuGet
lib\native\amd64\release\k4arecord.dll | GitHub Build | NuGet
lib\native\amd64\release\k4arecord.lib | GitHub Build | NuGet
lib\native\amd64\release\k4arecord.pdb | GitHub Build | NuGet
lib\netstandard2.0\release\Microsoft.AzureKinect.deps.json | GitHub Build | NuGet
lib\netstandard2.0\release\Microsoft.AzureKinect.dll | GitHub Build | NuGet
lib\netstandard2.0\release\Microsoft.AzureKinect.pdb | GitHub Build | NuGet
lib\netstandard2.0\release\Microsoft.AzureKinect.xml | GitHub Build | NuGet

### Ubuntu
File | Source | Package(s)
---- | ------ | ----------
\usr\bin\AzureKinectFirmwareTool | GitHub Build | libk4a-tools
\usr\include\k4a\k4a.h | GitHub Source | libk4a-tools
\usr\include\k4a\k4a.hpp | GitHub Source | libk4a-tools
\usr\include\k4a\k4atypes.h | GitHub Source | libk4a-tools
\usr\include\k4a\k4aversion.h | GitHub Source | libk4a-tools
\usr\include\k4a\k4a_export.h | GitHub Source | libk4a-tools
\usr\include\k4arecord\k4arecord_export.h | GitHub Source | libk4a-tools
\usr\include\k4arecord\playback.h | GitHub Source | libk4a-tools
\usr\include\k4arecord\record.h | GitHub Source | libk4a-tools
\usr\include\k4arecord\types.h | GitHub Source | libk4a-tools
\usr\lib\x86_64-linux-gnu\libdepthengine.so | Microsoft Internal | libk4a
\usr\lib\x86_64-linux-gnu\libdepthengine.so.1.0 | Microsoft Internal | libk4a
\usr\lib\x86_64-linux-gnu\libk4a.so | GitHub Build | libk4a
\usr\lib\x86_64-linux-gnu\libk4a.so.1 | GitHub Build | libk4a
\usr\lib\x86_64-linux-gnu\libk4a.so.1.1.0 | GitHub Build | libk4a
\usr\lib\x86_64-linux-gnu\libk4arecord.so | GitHub Build | libk4a
\usr\lib\x86_64-linux-gnu\libk4arecord.so.1 | GitHub Build | libk4a
\usr\lib\x86_64-linux-gnu\libk4arecord.so.1.1.0 | GitHub Build | libk4a
\usr\lib\x86_64-linux-gnu\cmake\k4a\k4aConfig.cmake | GitHub Build | libk4a-dev
\usr\lib\x86_64-linux-gnu\cmake\k4a\k4aConfigVersion.cmake | GitHub Build | libk4a-dev
\usr\lib\x86_64-linux-gnu\cmake\k4a\k4aTargets-relwithdebinfo.cmake | GitHub Build | libk4a-dev
\usr\lib\x86_64-linux-gnu\cmake\k4a\k4aTargets.cmake | GitHub Build | libk4a-dev
\usr\lib\x86_64-linux-gnu\cmake\k4arecord\k4arecordConfig.cmake | GitHub Build | libk4a-dev
\usr\lib\x86_64-linux-gnu\cmake\k4arecord\k4arecordConfigVersion.cmake | GitHub Build | libk4a-dev
\usr\lib\x86_64-linux-gnu\cmake\k4arecord\k4arecordTargets-relwithdebinfo.cmake | GitHub Build | libk4a-dev
\usr\lib\x86_64-linux-gnu\cmake\k4arecord\k4arecordTargets.cmake | GitHub Build | libk4a-dev
\usr\share\doc\libk4a1\LICENSE | Microsoft Internal | All
\usr\share\doc\libk4a1\REDIST | Microsoft Internal | All
\usr\share\doc\libk4a1\ThirdPartyNotices | Microsoft Internal | All
\usr\share\doc\libk4a1\version | Microsoft Internal | All
