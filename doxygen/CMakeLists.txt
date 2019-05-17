set(DOXYREST_PATH "d:/git/Azure-Kinect-Sensor-SDK/doxygen/doxyrest-1.1.1-a-windows-amd64")

set(DOXYREST_FRAME_DIR ${DOXYREST_PATH}/frame)

set(XML_DIR ${DOXYGEN_OUTPUT_DIRECTORY}/xml)
set(RST_DIR ${DOXYGEN_OUTPUT_DIRECTORY}/rst)
set(TMP_DIR ${DOXYGEN_OUTPUT_DIRECTORY}/tmp)
set(HTML_DIR ${DOXYGEN_OUTPUT_DIRECTORY}/doxyrest_html)
set(DOC_NAME "Azure Kinect Sensor SDK")


# configure_file(Doxyfile.in ${CMAKE_CURRENT_BINARY_DIR}/Doxyfile)

configure_file(
	conf.py.in
	${RST_DIR}/conf.py
	)

set(
	DOXYREST_CMD_LINE
	${XML_DIR}/index.xml
	-c ${CMAKE_CURRENT_LIST_DIR}/doxyrest-config.lua
	-o ${RST_DIR}/index.rst
	-F ${DOXYREST_FRAME_DIR}/cfamily
	-F ${DOXYREST_FRAME_DIR}/common
	-f index.rst.in
	)


add_custom_command(
    OUTPUT ${RST_DIR}/index.rst
    COMMAND ${DOXYREST_PATH}/bin/doxyrest.exe ${DOXYREST_CMD_LINE}
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    MAIN_DEPENDENCY ${XML_DIR}/index.xml
    )
add_custom_target(k4adocsrst ALL DEPENDS ${RST_DIR}/index.rst)

add_custom_command(
    OUTPUT ${HTML_DIR}/index.html
    COMMAND sphinx-build -b html ${RST_DIR} ${HTML_DIR} 
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    MAIN_DEPENDENCY ${RST_DIR}/index.rst
    )
add_custom_target(k4adocshtml ALL DEPENDS ${HTML_DIR}/index.html)


set(
	SPHINX_HTML_CMD_LINE
	-b html
	-d ${TMP_DIR}/sphinx-doctrees
	-c ${CMAKE_CURRENT_BINARY_DIR}
	${RST_DIR}
	${HTML_DIR}
	)

#. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .

set(
	CMD_LINE_VAR_LIST

	DOXYREST_CMD_LINE
	SPHINX_HTML_CMD_LINE
	)

foreach(_VAR ${CMD_LINE_VAR_LIST})
	string(
		REPLACE
		";" " "
		${_VAR}
		"${${_VAR}}"
		)
endforeach()

#. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .

