idf_component_register(SRCS "esp_lcd_co5300.c" "esp_lcd_co5300_spi.c" "esp_lcd_co5300_mipi.c"
                       INCLUDE_DIRS "include"
                       PRIV_INCLUDE_DIRS "priv_include"
                       REQUIRES "esp_lcd" "driver")

# vendor 化: 去掉 include(package_manager)/cu_pkg_define_version (依赖 espressif__cmake_utilities),
# 版本宏直接写死 (与 idf_component.yml 的 2.1.0 一致)
target_compile_definitions(${COMPONENT_LIB} PUBLIC
    ESP_LCD_CO5300_VER_MAJOR=2
    ESP_LCD_CO5300_VER_MINOR=1
    ESP_LCD_CO5300_VER_PATCH=0)
