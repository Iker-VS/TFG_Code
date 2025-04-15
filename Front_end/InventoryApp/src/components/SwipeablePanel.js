"use client"

import { useContext, useRef } from "react"
import { Animated, StyleSheet, View } from "react-native"
import { Swipeable, TouchableOpacity } from "react-native-gesture-handler"
import { Ionicons } from "@expo/vector-icons"
import { ThemeContext } from "../context/ThemeContext"

const SwipeablePanel = ({ children, onEdit, onDelete }) => {
  const { theme } = useContext(ThemeContext)
  const swipeableRef = useRef(null)

  const renderRightActions = (progress, dragX) => {
    const trans = dragX.interpolate({
      inputRange: [-100, 0],
      outputRange: [0, 100],
      extrapolate: "clamp",
    })

    return (
      <View style={styles.rightActionsContainer}>
        <Animated.View
          style={[
            styles.rightAction,
            {
              backgroundColor: theme.swipeDelete,
              transform: [{ translateX: trans }],
            },
          ]}
        >
          <TouchableOpacity
            style={styles.actionButton}
            onPress={() => {
              swipeableRef.current.close()
              onDelete()
            }}
          >
            <Ionicons name="trash-outline" size={24} color="#fff" />
          </TouchableOpacity>
        </Animated.View>
      </View>
    )
  }

  const renderLeftActions = (progress, dragX) => {
    const trans = dragX.interpolate({
      inputRange: [0, 100],
      outputRange: [-100, 0],
      extrapolate: "clamp",
    })

    return (
      <View style={styles.leftActionsContainer}>
        <Animated.View
          style={[
            styles.leftAction,
            {
              backgroundColor: theme.swipeEdit,
              transform: [{ translateX: trans }],
            },
          ]}
        >
          <TouchableOpacity
            style={styles.actionButton}
            onPress={() => {
              swipeableRef.current.close()
              onEdit()
            }}
          >
            <Ionicons name="create-outline" size={24} color="#fff" />
          </TouchableOpacity>
        </Animated.View>
      </View>
    )
  }

  return (
    <Swipeable
      ref={swipeableRef}
      renderRightActions={renderRightActions}
      renderLeftActions={renderLeftActions}
      friction={2}
      rightThreshold={40}
      leftThreshold={40}
    >
      {children}
    </Swipeable>
  )
}

const styles = StyleSheet.create({
  rightActionsContainer: {
    width: 80,
    flexDirection: "row",
  },
  leftActionsContainer: {
    width: 80,
    flexDirection: "row",
  },
  rightAction: {
    flex: 1,
    justifyContent: "center",
    alignItems: "center",
  },
  leftAction: {
    flex: 1,
    justifyContent: "center",
    alignItems: "center",
  },
  actionButton: {
    width: "100%",
    height: "100%",
    justifyContent: "center",
    alignItems: "center",
  },
})

export default SwipeablePanel
